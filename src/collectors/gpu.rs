use crate::models::{GpuMetrics, MetricSample};
use anyhow::{Context, Result};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIFactory1};

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
type NvmlDeviceGetHandleByIndex = unsafe extern "C" fn(u32, *mut *mut core::ffi::c_void) -> i32;
type NvmlDeviceGetName = unsafe extern "C" fn(*mut core::ffi::c_void, *mut u8, u32) -> i32;
type NvmlDeviceGetMemoryInfo = unsafe extern "C" fn(*mut core::ffi::c_void, *mut NvmlMemory) -> i32;
type NvmlDeviceGetUtilizationRates =
    unsafe extern "C" fn(*mut core::ffi::c_void, *mut NvmlUtilization) -> i32;
type NvmlDeviceGetTemperature = unsafe extern "C" fn(*mut core::ffi::c_void, i32, *mut u32) -> i32;

struct Nvml {
    _lib: libloading::Library,
    shutdown: NvmlShutdown,
    _get_handle: NvmlDeviceGetHandleByIndex,
    get_name: NvmlDeviceGetName,
    get_memory: NvmlDeviceGetMemoryInfo,
    get_utilization: NvmlDeviceGetUtilizationRates,
    get_temperature: NvmlDeviceGetTemperature,
    device: *mut core::ffi::c_void,
}

impl Nvml {
    fn load() -> Result<Self> {
        let candidates = ["nvml.dll", "C:\\Windows\\System32\\nvml.dll"];
        let mut last_error = None;
        for candidate in candidates {
            match unsafe { libloading::Library::new(candidate) } {
                Ok(lib) => {
                    let init: libloading::Symbol<NvmlInit> = unsafe { lib.get(b"nvmlInit_v2\0") }
                        .or_else(|_| unsafe { lib.get(b"nvmlInit\0") })
                        .context("nvmlInit symbol")?;
                    let status = unsafe { init() };
                    if status != 0 {
                        anyhow::bail!("nvmlInit failed: {status}");
                    }
                    let shutdown: NvmlShutdown =
                        *unsafe { lib.get(b"nvmlShutdown\0") }.context("nvmlShutdown")?;
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
                    let mut device = core::ptr::null_mut();
                    let status = unsafe { get_handle(0, &mut device) };
                    if status != 0 || device.is_null() {
                        anyhow::bail!("nvmlDeviceGetHandleByIndex(0) failed: {status}");
                    }
                    return Ok(Self {
                        _lib: lib,
                        shutdown,
                        _get_handle: get_handle,
                        get_name,
                        get_memory,
                        get_utilization,
                        get_temperature,
                        device,
                    });
                }
                Err(err) => last_error = Some(err),
            }
        }
        match last_error {
            Some(err) => Err(anyhow::anyhow!("nvml.dll unavailable: {err}")),
            None => Err(anyhow::anyhow!("nvml.dll unavailable")),
        }
    }

    fn read(&self) -> Option<GpuMetrics> {
        let mut name_buf = [0u8; 256];
        let name_status =
            unsafe { (self.get_name)(self.device, name_buf.as_mut_ptr(), name_buf.len() as u32) };
        let name = if name_status == 0 {
            cstr_to_string(&name_buf)
        } else {
            "NVIDIA GPU".to_string()
        };
        let mut memory = NvmlMemory {
            total: 0,
            free: 0,
            used: 0,
        };
        let memory_ok = unsafe { (self.get_memory)(self.device, &mut memory) } == 0;
        let mut utilization = NvmlUtilization { gpu: 0, memory: 0 };
        let usage = if unsafe { (self.get_utilization)(self.device, &mut utilization) } == 0 {
            Some(utilization.gpu as f32)
        } else {
            None
        };
        let mut temperature = 0u32;
        let temperature_c =
            if unsafe { (self.get_temperature)(self.device, 0, &mut temperature) } == 0 {
                Some(temperature as f32)
            } else {
                None
            };
        Some(GpuMetrics {
            name,
            usage_percent: usage,
            vram_used_bytes: if memory_ok { Some(memory.used) } else { None },
            vram_total_bytes: if memory_ok { Some(memory.total) } else { None },
            temperature_c,
            source: "NVML".to_string(),
        })
    }
}

unsafe impl Send for Nvml {}

impl Drop for Nvml {
    fn drop(&mut self) {
        unsafe { (self.shutdown)() };
    }
}

pub struct GpuCollector {
    nvml: Option<Nvml>,
    dxgi_name: String,
    dxgi_vram: Option<u64>,
    page_file: u64,
}

impl GpuCollector {
    pub fn new(total_physical_bytes: u64) -> Result<Self> {
        let (dxgi_name, dxgi_vram) =
            read_dxgi_adapter().unwrap_or_else(|_| ("Unknown GPU".to_string(), None));
        let nvml = match Nvml::load() {
            Ok(nvml) => Some(nvml),
            Err(err) => {
                tracing::info!("NVML unavailable, using DXGI fallback: {err}");
                None
            }
        };
        Ok(Self {
            nvml,
            dxgi_name,
            dxgi_vram,
            page_file: total_physical_bytes,
        })
    }

    pub fn read(&mut self) -> GpuMetrics {
        if let Some(nvml) = &self.nvml {
            if let Some(metrics) = nvml.read() {
                return metrics;
            }
        }
        GpuMetrics {
            name: self.dxgi_name.clone(),
            usage_percent: None,
            vram_used_bytes: None,
            vram_total_bytes: self.dxgi_vram.or(Some(self.page_file)),
            temperature_c: None,
            source: "DXGI (limited telemetry)".to_string(),
        }
    }
}

impl crate::collectors::Collector for GpuCollector {
    fn name(&self) -> &'static str {
        "gpu"
    }

    fn collect(&mut self, sample: &mut MetricSample) -> Result<()> {
        sample.gpu = self.read();
        Ok(())
    }
}

fn read_dxgi_adapter() -> Result<(String, Option<u64>)> {
    let factory: IDXGIFactory1 = unsafe { CreateDXGIFactory1() }.context("CreateDXGIFactory1")?;
    let adapter = unsafe { factory.EnumAdapters1(0) }.context("EnumAdapters1(0)")?;
    let desc = unsafe { adapter.GetDesc1() }.context("GetDesc1")?;
    let end = desc
        .Description
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(desc.Description.len());
    let name = String::from_utf16_lossy(&desc.Description[..end]);
    Ok((name, Some(desc.DedicatedVideoMemory as u64)))
}

fn cstr_to_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}
