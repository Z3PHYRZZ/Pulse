// language: Rust, file: gpu.rs, target: Windows 11 x64/ARM64, edition 2021
use crate::models::{GpuMetrics, GpuVendor, MetricSample};
use anyhow::{Context, Result};
use windows::core::PCWSTR;
use windows::Win32::Graphics::Dxgi::{
    CreateDXGIFactory1, IDXGIFactory1, DXGI_ADAPTER_FLAG_SOFTWARE,
};
use windows::Win32::System::Performance::{
    PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW,
    PdhOpenQueryW, PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY,
};

const GPU_ENGINE_COUNTER: &str = "\\GPU Engine(*)\\Utilization Percentage";

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct NvmlMemory {
    total: u64,
    free: u64,
    used: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct NvmlUtilization {
    gpu: u32,
    memory: u32,
}

type NvmlInit = unsafe extern "C" fn() -> i32;
type NvmlShutdown = unsafe extern "C" fn() -> i32;
type NvmlDeviceGetCount = unsafe extern "C" fn(*mut u32) -> i32;
type NvmlDeviceGetHandleByIndex = unsafe extern "C" fn(u32, *mut *mut core::ffi::c_void) -> i32;
type NvmlDeviceGetName = unsafe extern "C" fn(*mut core::ffi::c_void, *mut u8, u32) -> i32;
type NvmlDeviceGetMemoryInfo =
    unsafe extern "C" fn(*mut core::ffi::c_void, *mut NvmlMemory) -> i32;
type NvmlDeviceGetUtilizationRates =
    unsafe extern "C" fn(*mut core::ffi::c_void, *mut NvmlUtilization) -> i32;
type NvmlDeviceGetTemperature =
    unsafe extern "C" fn(*mut core::ffi::c_void, i32, *mut u32) -> i32;
type NvmlDeviceGetUuid = unsafe extern "C" fn(*mut core::ffi::c_void, *mut u8, u32) -> i32;

/// One NVML device handle plus its cached identity. NVML handles are only
/// valid while the library is loaded, so they live inside `Nvml`.
struct NvmlDevice {
    handle: *mut core::ffi::c_void,
    index: u32,
    uuid: String,
    name: String,
}

unsafe impl Send for NvmlDevice {}

struct Nvml {
    _lib: libloading::Library,
    shutdown: NvmlShutdown,
    get_count: NvmlDeviceGetCount,
    _get_handle: NvmlDeviceGetHandleByIndex,
    get_name: NvmlDeviceGetName,
    get_memory: NvmlDeviceGetMemoryInfo,
    get_utilization: NvmlDeviceGetUtilizationRates,
    get_temperature: NvmlDeviceGetTemperature,
    get_uuid: Option<NvmlDeviceGetUuid>,
    devices: Vec<NvmlDevice>,
}

unsafe impl Send for Nvml {}

impl Nvml {
    fn load() -> Result<Self> {
        // Load NVML only from the system directory by absolute path. A bare
        // "nvml.dll" would resolve through the DLL search order (application
        // directory first), letting a planted library execute in this process.
        let system_dir = std::env::var("SystemRoot")
            .map(|root| format!("{root}\\System32\\nvml.dll"))
            .unwrap_or_else(|_| "C:\\Windows\\System32\\nvml.dll".to_string());
        let lib = unsafe { libloading::Library::new(&system_dir) }
            .with_context(|| format!("nvml.dll unavailable at {system_dir}"))?;
        let init: libloading::Symbol<NvmlInit> = unsafe { lib.get(b"nvmlInit_v2\0") }
            .or_else(|_| unsafe { lib.get(b"nvmlInit\0") })
            .context("nvmlInit symbol")?;
        let status = unsafe { init() };
        if status != 0 {
            anyhow::bail!("nvmlInit failed: {status}");
        }
        let shutdown: NvmlShutdown =
            *unsafe { lib.get(b"nvmlShutdown\0") }.context("nvmlShutdown")?;
        let get_count: NvmlDeviceGetCount =
            *unsafe { lib.get(b"nvmlDeviceGetCount_v2\0") }
                .or_else(|_| unsafe { lib.get(b"nvmlDeviceGetCount\0") })
                .context("nvmlDeviceGetCount")?;
        let get_handle: NvmlDeviceGetHandleByIndex =
            *unsafe { lib.get(b"nvmlDeviceGetHandleByIndex_v2\0") }
                .or_else(|_| unsafe { lib.get(b"nvmlDeviceGetHandleByIndex\0") })
                .context("nvmlDeviceGetHandleByIndex")?;
        let get_name: NvmlDeviceGetName =
            *unsafe { lib.get(b"nvmlDeviceGetName\0") }.context("nvmlDeviceGetName")?;
        let get_memory: NvmlDeviceGetMemoryInfo =
            *unsafe { lib.get(b"nvmlDeviceGetMemoryInfo\0") }
                .or_else(|_| unsafe { lib.get(b"nvmlDeviceGetMemoryInfo_v2\0") })
                .context("nvmlDeviceGetMemoryInfo")?;
        let get_utilization: NvmlDeviceGetUtilizationRates =
            *unsafe { lib.get(b"nvmlDeviceGetUtilizationRates\0") }
                .context("nvmlDeviceGetUtilizationRates")?;
        let get_temperature: NvmlDeviceGetTemperature =
            *unsafe { lib.get(b"nvmlDeviceGetTemperature\0") }
                .context("nvmlDeviceGetTemperature")?;
        let get_uuid: Option<NvmlDeviceGetUuid> = unsafe { lib.get(b"nvmlDeviceGetUUID\0") }
            .ok()
            .map(|s| *s);

        let mut count = 0u32;
        if unsafe { get_count(&mut count) } != 0 || count == 0 {
            unsafe { shutdown() };
            anyhow::bail!("nvmlDeviceGetCount reported no devices");
        }
        let mut devices = Vec::with_capacity(count as usize);
        for index in 0..count {
            let mut handle = core::ptr::null_mut();
            if unsafe { get_handle(index, &mut handle) } != 0 || handle.is_null() {
                continue;
            }
            let mut name_buf = [0u8; 256];
            let name = if unsafe { get_name(handle, name_buf.as_mut_ptr(), name_buf.len() as u32) }
                == 0
            {
                cstr_to_string(&name_buf)
            } else {
                format!("NVIDIA GPU {index}")
            };
            let uuid = if let Some(get_uuid) = get_uuid {
                let mut uuid_buf = [0u8; 80];
                if unsafe { get_uuid(handle, uuid_buf.as_mut_ptr(), uuid_buf.len() as u32) } == 0 {
                    cstr_to_string(&uuid_buf)
                } else {
                    String::new()
                }
            } else {
                String::new()
            };
            devices.push(NvmlDevice {
                handle,
                index,
                uuid,
                name,
            });
        }
        if devices.is_empty() {
            unsafe { shutdown() };
            anyhow::bail!("no NVML device handles could be opened");
        }
        Ok(Self {
            _lib: lib,
            shutdown,
            get_count,
            _get_handle: get_handle,
            get_name,
            get_memory,
            get_utilization,
            get_temperature,
            get_uuid,
            devices,
        })
    }

    /// Try to bind an NVML device to a DXGI adapter. NVML exposes a stable
    /// UUID and (on modern drivers) a PCI bus id; DXGI exposes an LUID. There
    /// is no shared identifier across both APIs, so bind by index order for
    /// NVIDIA adapters — this is correct in the overwhelming majority of
    /// single-vendor systems. UUID is cached for diagnostics only.
    fn device_for_nvidia_adapter(&self, nvidia_ordinal: usize) -> Option<&NvmlDevice> {
        self.devices.get(nvidia_ordinal)
    }

    fn read(&self, device: &NvmlDevice, base: &GpuMetrics) -> GpuMetrics {
        let mut memory = NvmlMemory {
            total: 0,
            free: 0,
            used: 0,
        };
        let memory_ok = unsafe { (self.get_memory)(device.handle, &mut memory) } == 0;
        let mut utilization = NvmlUtilization { gpu: 0, memory: 0 };
        let mut usage_percent = None;
        if unsafe { (self.get_utilization)(device.handle, &mut utilization) } == 0 {
            usage_percent = Some(utilization.gpu as f32);
        }
        let mut temperature = 0u32;
        let temperature_c =
            if unsafe { (self.get_temperature)(device.handle, 0, &mut temperature) } == 0 {
                Some(temperature as f32)
            } else {
                None
            };
        GpuMetrics {
            name: if base.name.is_empty() {
                device.name.clone()
            } else {
                base.name.clone()
            },
            usage_percent,
            vram_used_bytes: if memory_ok { Some(memory.used) } else { None },
            vram_total_bytes: if memory_ok {
                Some(memory.total)
            } else {
                base.vram_total_bytes
            },
            temperature_c,
            source: "NVML".to_string(),
            ..base.clone()
        }
    }
}

impl Drop for Nvml {
    fn drop(&mut self) {
        unsafe { (self.shutdown)() };
    }
}

/// PDH GPU Engine wildcard counter, sampled for non-NVIDIA utilization.
/// The counter instance names embed the physical adapter name, so summing
/// all instances whose name contains the adapter name gives per-adapter load.
struct GpuEngineQuery {
    query: PDH_HQUERY,
    counter: PDH_HCOUNTER,
}

unsafe impl Send for GpuEngineQuery {}

impl GpuEngineQuery {
    fn new() -> Option<Self> {
        let mut query = PDH_HQUERY::default();
        if unsafe { PdhOpenQueryW(PCWSTR::null(), 0, &mut query) } != 0 {
            return None;
        }
        let wide: Vec<u16> = GPU_ENGINE_COUNTER
            .encode_utf16()
            .chain(std::iter::once(0))
            .collect();
        let mut counter = PDH_HCOUNTER::default();
        if unsafe {
            PdhAddEnglishCounterW(query, PCWSTR::from_raw(wide.as_ptr()), 0, &mut counter)
        } != 0
        {
            unsafe { PdhCloseQuery(query) };
            return None;
        }
        // Prime the counter; rate counters need two collections.
        for _ in 0..2 {
            if unsafe { PdhCollectQueryData(query) } != 0 {
                unsafe { PdhCloseQuery(query) };
                return None;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        Some(Self { query, counter })
    }

    /// Returns (instance name, percentage) pairs from the latest collection.
    fn values(&mut self) -> Vec<(String, f32)> {
        if unsafe { PdhCollectQueryData(self.query) } != 0 {
            return Vec::new();
        }
        let mut buffer_size = 0u32;
        let mut item_count = 0u32;
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &mut buffer_size,
                &mut item_count,
                None,
            )
        };
        // PDH_ERROR_INSUFFICIENT_BUFFER (0x800007D2) is expected on the size probe.
        if status != 0x800007D2 || item_count == 0 || buffer_size == 0 {
            return Vec::new();
        }
        let word_len = (buffer_size as usize) / 2;
        let mut buffer = vec![0u16; word_len + 2];
        let mut actual_size = buffer_size;
        let mut actual_count = 0u32;
        let status = unsafe {
            PdhGetFormattedCounterArrayW(
                self.counter,
                PDH_FMT_DOUBLE,
                &mut actual_size,
                &mut actual_count,
                Some(buffer.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W),
            )
        };
        if status != 0 {
            return Vec::new();
        }
        let items = buffer.as_ptr() as *const PDH_FMT_COUNTERVALUE_ITEM_W;
        let mut out = Vec::with_capacity(actual_count as usize);
        for index in 0..actual_count as usize {
            let item = unsafe { &*items.add(index) };
            if item.FmtValue.CStatus != 0 || item.szName.0.is_null() {
                continue;
            }
            let name = unsafe { windows::core::PWSTR(item.szName.0).to_string() }
                .unwrap_or_default();
            let value = unsafe { item.FmtValue.Anonymous.doubleValue } as f32;
            if value.is_finite() {
                out.push((name, value));
            }
        }
        out
    }
}

impl Drop for GpuEngineQuery {
    fn drop(&mut self) {
        unsafe { PdhCloseQuery(self.query) };
    }
}

pub struct GpuCollector {
    adapters: Vec<GpuMetrics>,
    nvml: Option<Nvml>,
    engine_query: Option<GpuEngineQuery>,
    page_file: u64,
}

impl GpuCollector {
    pub fn new(total_physical_bytes: u64) -> Result<Self> {
        let mut adapters = read_dxgi_adapters().unwrap_or_else(|err| {
            tracing::warn!("DXGI adapter enumeration failed: {err}");
            Vec::new()
        });
        if adapters.is_empty() {
            adapters.push(GpuMetrics {
                name: "Unknown GPU".to_string(),
                vendor: GpuVendor::Unknown.as_str().to_string(),
                source: "DXGI (no adapters)".to_string(),
                ..GpuMetrics::default()
            });
        }
        let nvml = match Nvml::load() {
            Ok(nvml) => Some(nvml),
            Err(err) => {
                tracing::info!("NVML unavailable, using DXGI + PDH fallback: {err}");
                None
            }
        };
        let mut nvidia_ordinal = 0usize;
        if let Some(nvml) = &nvml {
            for adapter in adapters.iter_mut() {
                if adapter.vendor == GpuVendor::Nvidia.as_str() {
                    if let Some(device) = nvml.device_for_nvidia_adapter(nvidia_ordinal) {
                        *adapter = nvml.read(device, adapter);
                    }
                    nvidia_ordinal += 1;
                }
            }
        }
        let engine_query = GpuEngineQuery::new();
        if engine_query.is_none() {
            tracing::info!("GPU Engine PDH counter unavailable; utilization may be n/a");
        }
        Ok(Self {
            adapters,
            nvml,
            engine_query,
            page_file: total_physical_bytes,
        })
    }

    pub fn read(&mut self) -> Vec<GpuMetrics> {
        let engine_values = self
            .engine_query
            .as_mut()
            .map(|query| query.values())
            .unwrap_or_default();
        let nvml = self.nvml.as_ref();
        let mut nvidia_ordinal = 0usize;
        self.adapters
            .iter()
            .map(|base| {
                let mut metrics = base.clone();
                if metrics.vram_total_bytes.is_none() {
                    metrics.vram_total_bytes = Some(self.page_file);
                    metrics.source = "DXGI (limited telemetry)".to_string();
                }
                if metrics.vendor == GpuVendor::Nvidia.as_str() {
                    if let Some(nvml) = nvml {
                        if let Some(device) = nvml.device_for_nvidia_adapter(nvidia_ordinal) {
                            metrics = nvml.read(device, &metrics);
                        }
                    }
                    nvidia_ordinal += 1;
                }
                if metrics.usage_percent.is_none() && !engine_values.is_empty() {
                    let needle = adapter_match_key(&metrics.name);
                    let matched: Vec<&f32> = engine_values
                        .iter()
                        .filter(|(instance, _)| instance.to_lowercase().contains(&needle))
                        .map(|(_, value)| value)
                        .collect();
                    // On a single-adapter system, PDH instance names do not carry
                    // the adapter name; attribute the full engine set to it.
                    let values = if matched.is_empty() && self.adapters.len() == 1 {
                        engine_values.iter().map(|(_, v)| v).collect::<Vec<_>>()
                    } else {
                        matched
                    };
                    let sum: f32 = values.iter().copied().sum();
                    if sum > 0.0 {
                        // GPU Engine exposes several sub-engines (3D, video decode,
                        // copy); their percentages are independent, so cap the
                        // aggregate at 100% rather than reporting >100%.
                        metrics.usage_percent = Some(sum.min(100.0));
                        if metrics.source.starts_with("DXGI") {
                            metrics.source = "PDH GPU Engine".to_string();
                        }
                    }
                }
                metrics
            })
            .collect()
    }

    /// Primary GPU for the backwards-compatible `MetricSample.gpu` field:
    /// first discrete adapter, otherwise the first adapter.
    pub fn primary(&self) -> GpuMetrics {
        self.adapters
            .iter()
            .find(|adapter| !adapter.is_integrated)
            .or_else(|| self.adapters.first())
            .cloned()
            .unwrap_or_default()
    }
}

impl crate::collectors::Collector for GpuCollector {
    fn name(&self) -> &'static str {
        "gpu"
    }

    fn collect(&mut self, sample: &mut MetricSample) -> Result<()> {
        let all = self.read();
        if let Some(primary) = all.iter().find(|gpu| !gpu.is_integrated) {
            sample.gpu = primary.clone();
        } else if let Some(first) = all.first() {
            sample.gpu = first.clone();
        }
        sample.gpus = all;
        Ok(())
    }
}

fn adapter_match_key(name: &str) -> String {
    // PDH GPU Engine instance names contain the adapter name, often with
    // vendor prefix and suffix. Match on the longest distinctive token.
    let lower = name.to_lowercase();
    let cleaned = lower
        .replace("(r)", "")
        .replace("(tm)", "")
        .replace("(c)", "")
        .replace("  ", " ");
    cleaned.trim().to_string()
}

fn read_dxgi_adapters() -> Result<Vec<GpuMetrics>> {
    let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }.context("CreateDXGIFactory1")?;
    let mut adapters = Vec::new();
    let mut index = 0u32;
    loop {
        let adapter = match unsafe { factory.EnumAdapters1(index) } {
            Ok(adapter) => adapter,
            Err(_) => break,
        };
        let desc = match unsafe { adapter.GetDesc1() } {
            Ok(desc) => desc,
            Err(_) => break,
        };
        let end = desc
            .Description
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(desc.Description.len());
        let name = String::from_utf16_lossy(&desc.Description[..end]);
        let vendor = GpuVendor::from_pci_vendor_id(desc.VendorId);
        let luid = desc.AdapterLuid;
        let id = format!("{:08X}-{:08X}", luid.HighPart as u32, luid.LowPart);
        let is_integrated = desc.Flags & (DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0
            || vendor == GpuVendor::Intel
            || vendor == GpuVendor::Qualcomm
            || (vendor == GpuVendor::Amd && name.to_lowercase().contains("graphics"))
            || name.to_lowercase().contains("uhd graphics")
            || name.to_lowercase().contains("iris");
        let architecture = match vendor {
            GpuVendor::Qualcomm if name.to_lowercase().contains("adreno") => "Adreno".to_string(),
            GpuVendor::Nvidia => "NVIDIA".to_string(),
            GpuVendor::Intel => "Intel Graphics".to_string(),
            GpuVendor::Amd => "AMD Graphics".to_string(),
            _ => String::new(),
        };
        adapters.push(GpuMetrics {
            id,
            name,
            vendor: vendor.as_str().to_string(),
            vendor_id: desc.VendorId,
            device_id: desc.DeviceId,
            architecture,
            is_integrated,
            vram_total_bytes: Some(desc.DedicatedVideoMemory as u64),
            dedicated_system_bytes: Some(desc.DedicatedSystemMemory as u64),
            shared_system_bytes: Some(desc.SharedSystemMemory as u64),
            source: "DXGI (limited telemetry)".to_string(),
            ..GpuMetrics::default()
        });
        index += 1;
        if index >= 64 {
            break;
        }
    }
    Ok(adapters)
}

fn cstr_to_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}
