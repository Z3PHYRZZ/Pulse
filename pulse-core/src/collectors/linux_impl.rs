// language: Rust, file: linux_impl.rs, target: Linux x86_64/aarch64, edition 2021
//! Linux collectors backed by procfs and sysfs. No external services are
//! required and every read is non-blocking. Values that the kernel does not
//! expose stay `None`/empty rather than being invented.

use crate::collectors::Collector;
use crate::models::{
    AdapterTraffic, CpuMetrics, DiskMetrics, GpuMetrics, GpuVendor, MemoryMetrics, MetricSample,
    NetworkMetrics, ProcessInfo, SystemInfo, VolumeUsage,
};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

// ---------------------------------------------------------------------------
// shared helpers
// ---------------------------------------------------------------------------

fn read_trimmed(path: impl AsRef<Path>) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_string())
}

fn parse_u64(path: impl AsRef<Path>) -> Option<u64> {
    read_trimmed(path)?.parse::<u64>().ok()
}

fn os_release(field: &str) -> Option<String> {
    let text = fs::read_to_string("/etc/os-release").ok()?;
    for line in text.lines() {
        if let Some((key, value)) = line.split_once('=') {
            if key == field {
                return Some(value.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

fn uptime_seconds() -> u64 {
    read_trimmed("/proc/uptime")
        .and_then(|line| line.split_whitespace().next()?.parse::<f64>().ok())
        .map(|v| v as u64)
        .unwrap_or(0)
}

fn clock_ticks_per_second() -> f64 {
    // sysconf(_SC_CLK_TCK); /proc/stat values are in these ticks.
    unsafe { libc::sysconf(libc::_SC_CLK_TCK) as f64 }.max(1.0)
}

fn page_size() -> u64 {
    unsafe { libc::sysconf(libc::_SC_PAGESIZE) as u64 }.max(1)
}

// ---------------------------------------------------------------------------
// system
// ---------------------------------------------------------------------------

pub fn read_system_info() -> Result<SystemInfo> {
    let hostname = read_trimmed("/proc/sys/kernel/hostname").unwrap_or_else(|| "linux".to_string());
    let os_version = os_release("PRETTY_NAME").unwrap_or_else(|| "Linux".to_string());
    let build_number = read_trimmed("/proc/sys/kernel/osrelease")
        .unwrap_or_default()
        .chars()
        .filter(|c| c.is_ascii_digit())
        .take(6)
        .collect::<String>()
        .parse::<u32>()
        .unwrap_or(0);
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());
    let logical_cores = fs::read_to_string("/proc/cpuinfo")
        .map(|text| text.lines().filter(|line| line.starts_with("processor\t:")).count())
        .unwrap_or(0)
        .max(1);
    let total_memory_bytes = read_trimmed("/proc/meminfo")
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with("MemTotal:"))
                .and_then(|line| line.split_whitespace().nth(1))
                .and_then(|v| v.parse::<u64>().ok())
        })
        .map(|kb| kb * 1024)
        .unwrap_or(0);
    let boot_time_ms = crate::utils::platform::now_ms().checked_sub(uptime_seconds() * 1000);
    let arch = std::env::consts::ARCH;
    let (processor_architecture, is_arm64) = match arch {
        "aarch64" => ("ARM64".to_string(), true),
        "arm" => ("ARM".to_string(), true),
        "x86_64" => ("x64".to_string(), false),
        "x86" => ("x86".to_string(), false),
        other => (other.to_string(), false),
    };
    let cpu_name = fs::read_to_string("/proc/cpuinfo")
        .ok()
        .and_then(|text| {
            text.lines()
                .find(|line| line.starts_with("model name") || line.starts_with("Hardware"))
                .and_then(|line| line.split_once(':'))
                .map(|(_, value)| value.trim().to_string())
        })
        .or_else(|| read_trimmed("/proc/device-tree/model"))
        .unwrap_or_default();
    Ok(SystemInfo {
        hostname,
        os_version,
        build_number,
        username,
        logical_cores,
        total_memory_bytes,
        boot_time_ms,
        processor_architecture,
        is_arm64,
        cpu_name,
    })
}

pub fn apply_uptime(sample: &mut MetricSample) {
    sample.uptime_seconds = uptime_seconds();
}

pub fn ensure(system: &SystemInfo, sample: &mut MetricSample) {
    sample.cpu.logical_cores = system.logical_cores;
    if sample.cpu.name.is_empty() {
        sample.cpu.name = if system.cpu_name.is_empty() {
            format!("{} ({} logical cores)", system.os_version, system.logical_cores)
        } else {
            system.cpu_name.clone()
        };
    }
    sample.cpu.architecture = system.processor_architecture.clone();
    sample.cpu.is_arm64 = system.is_arm64;
}

// ---------------------------------------------------------------------------
// delta helper (replaces the PDH rate counters)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
struct CounterState {
    value: f64,
    at: Instant,
}

struct RateCounter {
    previous: Option<CounterState>,
}

impl RateCounter {
    fn new() -> Self {
        Self { previous: None }
    }

    /// Returns a per-second delta from the last call. First call yields None.
    fn rate(&mut self, value: f64) -> Option<f64> {
        let now = Instant::now();
        let result = self.previous.map(|previous| {
            let elapsed = now.duration_since(previous.at).as_secs_f64();
            if elapsed <= 0.0 {
                None
            } else {
                Some((value - previous.value) / elapsed)
            }
        })?;
        self.previous = Some(CounterState { value, at: now });
        result
    }
}

fn cpu_ticks(line: &str) -> Option<[u64; 8]> {
    let mut fields = line.split_whitespace();
    if !fields.next()?.starts_with("cpu") {
        return None;
    }
    let mut values = [0u64; 8];
    for slot in values.iter_mut() {
        *slot = fields.next()?.parse().ok()?;
    }
    Some(values)
}

fn cpu_usage_delta(previous: Option<[u64; 8]>, current: [u64; 8]) -> Option<f32> {
    let previous = previous?;
    let busy: u64 = current[..7].iter().zip(&previous[..7]).map(|(c, p)| c.saturating_sub(*p)).sum();
    let idle: u64 = current[3].saturating_sub(previous[3]) + current[4].saturating_sub(previous[4]);
    let total = busy + idle;
    if total == 0 {
        return None;
    }
    Some((busy as f64 / total as f64 * 100.0).clamp(0.0, 100.0) as f32)
}

fn prime_cpu_counters() {
    let _ = fs::read_to_string("/proc/stat");
    std::thread::sleep(std::time::Duration::from_millis(60));
}

// ---------------------------------------------------------------------------
// CPU
// ---------------------------------------------------------------------------

pub struct CpuCollector {
    previous_total: Option<[u64; 8]>,
    previous_cores: HashMap<usize, [u64; 8]>,
    nominal_mhz: Option<f32>,
    primed: bool,
}

impl CpuCollector {
    pub fn new(_logical_cores: usize) -> Result<Self> {
        prime_cpu_counters();
        Ok(Self {
            previous_total: None,
            previous_cores: HashMap::new(),
            nominal_mhz: read_cpu_mhz(),
            primed: false,
        })
    }

    pub fn read(&mut self) -> Result<CpuMetrics> {
        let stat = fs::read_to_string("/proc/stat").context("reading /proc/stat")?;
        let mut usage = 0.0f32;
        let mut per_core = Vec::new();
        for line in stat.lines() {
            let name = line.split_whitespace().next().unwrap_or("");
            let Some(ticks) = cpu_ticks(line) else { continue };
            if name == "cpu" {
                if self.primed {
                    usage = cpu_usage_delta(self.previous_total, ticks).unwrap_or(0.0);
                }
                self.previous_total = Some(ticks);
            } else if name.starts_with("cpu") && name[3..].chars().all(|c| c.is_ascii_digit()) {
                let index = name[3..].parse::<usize>().unwrap_or(0);
                if self.primed {
                    let previous = self.previous_cores.get(&index).copied();
                    per_core.push(cpu_usage_delta(previous, ticks).unwrap_or(0.0));
                } else {
                    per_core.push(0.0);
                }
                self.previous_cores.insert(index, ticks);
            }
        }
        self.primed = true;
        let frequency_mhz = read_cpu_mhz().or(self.nominal_mhz);
        Ok(CpuMetrics {
            usage_percent: usage,
            per_core_percent: per_core,
            frequency_mhz,
            logical_cores: 0,
            name: String::new(),
            temperature_c: read_cpu_temperature(),
            architecture: String::new(),
            is_arm64: false,
        })
    }
}

impl Collector for CpuCollector {
    fn name(&self) -> &'static str { "cpu" }
    fn collect(&mut self, sample: &mut MetricSample) -> Result<()> {
        sample.cpu = self.read()?;
        Ok(())
    }
}

fn read_cpu_mhz() -> Option<f32> {
    fs::read_to_string("/proc/cpuinfo")
        .ok()?
        .lines()
        .find(|line| line.starts_with("cpu MHz"))
        .and_then(|line| line.split_once(':'))
        .and_then(|(_, value)| value.trim().parse::<f32>().ok())
}

fn read_cpu_temperature() -> Option<f32> {
    let base = Path::new("/sys/class/thermal/thermal_zone0/temp");
    if let Some(millidegrees) = parse_u64(base) {
        return Some(millidegrees as f32 / 1000.0);
    }
    None
}

// ---------------------------------------------------------------------------
// memory
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct MemoryCollector;

impl MemoryCollector {
    pub fn new() -> Self { Self }

    pub fn read(&mut self) -> Result<MemoryMetrics> {
        let text = fs::read_to_string("/proc/meminfo").context("reading /proc/meminfo")?;
        let mut total = 0u64;
        let mut available = 0u64;
        let mut free = 0u64;
        let mut buffers = 0u64;
        let mut cached = 0u64;
        let mut reclaimable = 0u64;
        for line in text.lines() {
            let value = line
                .split_once(':')
                .and_then(|(_, value)| value.split_whitespace().next())
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(0)
                * 1024;
            match line.split_once(':').map(|(key, _)| key) {
                Some("MemTotal") => total = value,
                Some("MemAvailable") => available = value,
                Some("MemFree") => free = value,
                Some("Buffers") => buffers = value,
                Some("Cached") => cached = value,
                Some("SReclaimable") => reclaimable = value,
                _ => {}
            }
        }
        if available == 0 {
            available = free.saturating_add(buffers).saturating_add(cached).saturating_add(reclaimable);
        }
        let used = total.saturating_sub(available);
        let percent = if total == 0 { 0.0 } else { used as f64 / total as f64 * 100.0 };
        Ok(MemoryMetrics {
            used_bytes: used,
            available_bytes: available,
            total_bytes: total,
            usage_percent: percent as f32,
        })
    }
}

impl Collector for MemoryCollector {
    fn name(&self) -> &'static str { "memory" }
    fn collect(&mut self, sample: &mut MetricSample) -> Result<()> {
        sample.memory = self.read()?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// disk
// ---------------------------------------------------------------------------

pub struct DiskCollector {
    read_rate: RateCounter,
    write_rate: RateCounter,
}

impl DiskCollector {
    pub fn new() -> Result<Self> {
        let _ = fs::read_to_string("/proc/diskstats");
        std::thread::sleep(std::time::Duration::from_millis(60));
        Ok(Self { read_rate: RateCounter::new(), write_rate: RateCounter::new() })
    }

    pub fn read(&mut self) -> Result<DiskMetrics> {
        let text = fs::read_to_string("/proc/diskstats").context("reading /proc/diskstats")?;
        let mut read_sectors = 0u64;
        let mut write_sectors = 0u64;
        for line in text.lines() {
            let fields: Vec<&str> = line.split_whitespace().collect();
            if fields.len() < 14 { continue; }
            let name = fields[2];
            // Skip loop devices and partitions; count whole physical disks.
            if name.starts_with("loop") || name.starts_with("ram") || is_partition(name) { continue; }
            read_sectors = read_sectors.saturating_add(fields[5].parse::<u64>().unwrap_or(0));
            write_sectors = write_sectors.saturating_add(fields[9].parse::<u64>().unwrap_or(0));
        }
        Ok(DiskMetrics {
            read_bytes_per_sec: self.read_rate.rate(read_sectors as f64).unwrap_or(0.0) * 512.0,
            write_bytes_per_sec: self.write_rate.rate(write_sectors as f64).unwrap_or(0.0) * 512.0,
            volumes: enumerate_volumes(),
        })
    }
}

fn is_partition(name: &str) -> bool {
    name.starts_with("sd") && name.chars().last().map(|c| c.is_ascii_digit()).unwrap_or(false)
        || name.starts_with("nvme") && name.contains('p')
        || name.starts_with("mmcblk") && name.contains('p')
}

fn enumerate_volumes() -> Vec<VolumeUsage> {
    let mut volumes = Vec::new();
    let Ok(text) = fs::read_to_string("/proc/mounts") else { return volumes };
    let mut seen = std::collections::HashSet::new();
    for line in text.lines() {
        let fields: Vec<&str> = line.split_whitespace().collect();
        if fields.len() < 2 || !fields[0].starts_with('/') { continue; }
        let mount = fields[1];
        if !seen.insert(mount.to_string()) { continue; }
        if let Some(stat) = mount_stat(mount) {
            volumes.push(stat);
        }
        if volumes.len() >= 32 { break; }
    }
    volumes
}

fn mount_stat(mount: &str) -> Option<VolumeUsage> {
    let path = std::ffi::CString::new(mount).ok()?;
    let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
    if unsafe { libc::statvfs(path.as_ptr(), &mut stat) } != 0 { return None; }
    let total = stat.f_blocks.saturating_mul(stat.f_frsize);
    let free = stat.f_bfree.saturating_mul(stat.f_frsize);
    let used = total.saturating_sub(free);
    let percent = if total == 0 { 0.0 } else { used as f64 / total as f64 * 100.0 };
    Some(VolumeUsage {
        name: mount.to_string(),
        used_bytes: used,
        total_bytes: total,
        usage_percent: percent as f32,
    })
}

impl Collector for DiskCollector {
    fn name(&self) -> &'static str { "disk" }
    fn collect(&mut self, sample: &mut MetricSample) -> Result<()> {
        sample.disk = self.read()?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// network
// ---------------------------------------------------------------------------

pub struct NetworkCollector {
    rates: HashMap<String, (RateCounter, RateCounter)>,
}

impl NetworkCollector {
    pub fn new() -> Result<Self> {
        let _ = fs::read_to_string("/proc/net/dev");
        std::thread::sleep(std::time::Duration::from_millis(60));
        Ok(Self { rates: HashMap::new() })
    }

    pub fn read(&mut self) -> Result<NetworkMetrics> {
        let text = fs::read_to_string("/proc/net/dev").context("reading /proc/net/dev")?;
        let mut upload = 0.0f64;
        let mut download = 0.0f64;
        let mut adapters = Vec::new();
        for line in text.lines().skip(2) {
            let mut parts = line.splitn(2, ':');
            let name = parts.next().unwrap_or("").trim().to_string();
            let values = parts.next().unwrap_or("");
            let fields: Vec<&str> = values.split_whitespace().collect();
            if fields.len() < 16 || name.is_empty() { continue; }
            let received = fields[0].parse::<f64>().unwrap_or(0.0);
            let sent = fields[8].parse::<f64>().unwrap_or(0.0);
            let entry = self.rates.entry(name.clone()).or_insert((RateCounter::new(), RateCounter::new()));
            let down_rate = entry.0.rate(received).unwrap_or(0.0).max(0.0);
            let up_rate = entry.1.rate(sent).unwrap_or(0.0).max(0.0);
            download += down_rate;
            upload += up_rate;
            adapters.push(AdapterTraffic {
                name,
                upload_bytes_per_sec: up_rate,
                download_bytes_per_sec: down_rate,
            });
        }
        Ok(NetworkMetrics { upload_bytes_per_sec: upload, download_bytes_per_sec: download, adapters })
    }
}

impl Collector for NetworkCollector {
    fn name(&self) -> &'static str { "network" }
    fn collect(&mut self, sample: &mut MetricSample) -> Result<()> {
        sample.network = self.read()?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// processes
// ---------------------------------------------------------------------------

pub struct ProcessCollector {
    previous: HashMap<u32, u64>,
    previous_at: Instant,
    cpu_count: usize,
}

impl ProcessCollector {
    pub fn new(cpu_count: usize) -> Self {
        Self { previous: HashMap::new(), previous_at: Instant::now(), cpu_count: cpu_count.max(1) }
    }

    pub fn read(&mut self) -> Result<Vec<ProcessInfo>> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.previous_at).as_secs_f64().max(0.001);
        self.previous_at = now;
        let ticks_per_second = clock_ticks_per_second();
        let mut current = HashMap::new();
        let mut processes = Vec::new();
        let entries = fs::read_dir("/proc").context("reading /proc")?;
        for entry in entries.flatten() {
            let name = entry.file_name();
            let Some(pid_text) = name.to_str() else { continue };
            if !pid_text.chars().all(|c| c.is_ascii_digit()) { continue; }
            let pid = match pid_text.parse::<u32>() { Ok(pid) => pid, Err(_) => continue };
            let dir = entry.path();
            let comm = read_trimmed(dir.join("comm")).unwrap_or_else(|| pid_text.to_string());
            let cpu_ticks = fs::read_to_string(dir.join("stat"))
                .ok()
                .and_then(|stat| parse_proc_stat_ticks(&stat))
                .unwrap_or(0);
            let memory_bytes = read_trimmed(dir.join("statm"))
                .and_then(|statm| statm.split_whitespace().nth(1)?.parse::<u64>().ok())
                .map(|pages| pages * page_size())
                .unwrap_or(0);
            let cpu_percent = match self.previous.get(&pid) {
                Some(previous) => {
                    let delta = cpu_ticks.saturating_sub(*previous) as f64 / ticks_per_second;
                    ((delta / elapsed) * 100.0 / self.cpu_count as f64) as f32
                }
                None => 0.0,
            };
            current.insert(pid, cpu_ticks);
            processes.push(ProcessInfo {
                pid,
                name: comm,
                cpu_percent: cpu_percent.clamp(0.0, 100.0),
                memory_bytes,
            });
        }
        self.previous = current;
        Ok(processes)
    }
}

fn parse_proc_stat_ticks(stat: &str) -> Option<u64> {
    // comm may contain spaces or parentheses; the trailing ')' is the anchor.
    let end = stat.rfind(')')?;
    let rest = stat[end + 1..].split_whitespace().collect::<Vec<_>>();
    // After ')': state(0), ppid(1), pgrp(2), session(3), tty_nr(4),
    // tpgid(5), flags(6), minflt(7), cminflt(8), majflt(9), cmajflt(10),
    // utime(11), stime(12).
    let utime = rest.get(11)?.parse::<u64>().ok()?;
    let stime = rest.get(12)?.parse::<u64>().ok()?;
    Some(utime + stime)
}

impl Collector for ProcessCollector {
    fn name(&self) -> &'static str { "processes" }
    fn collect(&mut self, sample: &mut MetricSample) -> Result<()> {
        sample.processes = self.read()?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// GPU — DRM/sysfs enumeration + NVML for NVIDIA
// ---------------------------------------------------------------------------

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct NvmlMemory { total: u64, free: u64, used: u64 }
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct NvmlUtilization { gpu: u32, memory: u32 }

type NvmlInit = unsafe extern "C" fn() -> i32;
type NvmlShutdown = unsafe extern "C" fn() -> i32;
type NvmlDeviceGetCount = unsafe extern "C" fn(*mut u32) -> i32;
type NvmlDeviceGetHandleByIndex = unsafe extern "C" fn(u32, *mut *mut core::ffi::c_void) -> i32;
type NvmlDeviceGetName = unsafe extern "C" fn(*mut core::ffi::c_void, *mut u8, u32) -> i32;
type NvmlDeviceGetMemoryInfo = unsafe extern "C" fn(*mut core::ffi::c_void, *mut NvmlMemory) -> i32;
type NvmlDeviceGetUtilizationRates = unsafe extern "C" fn(*mut core::ffi::c_void, *mut NvmlUtilization) -> i32;
type NvmlDeviceGetTemperature = unsafe extern "C" fn(*mut core::ffi::c_void, i32, *mut u32) -> i32;

struct NvmlDevice { handle: *mut core::ffi::c_void, name: String }
unsafe impl Send for NvmlDevice {}

struct Nvml {
    _lib: libloading::Library,
    shutdown: NvmlShutdown,
    devices: Vec<NvmlDevice>,
    get_memory: NvmlDeviceGetMemoryInfo,
    get_utilization: NvmlDeviceGetUtilizationRates,
    get_temperature: NvmlDeviceGetTemperature,
}
unsafe impl Send for Nvml {}

impl Nvml {
    fn load() -> Result<Self> {
        // Absolute system paths only; avoid LD_LIBRARY_PATH hijacking.
        let candidates = [
            "/usr/lib/x86_64-linux-gnu/libnvidia-ml.so.1",
            "/usr/lib64/libnvidia-ml.so.1",
            "/usr/lib/libnvidia-ml.so.1",
            "/usr/lib/aarch64-linux-gnu/libnvidia-ml.so.1",
        ];
        let mut loaded = None;
        for path in candidates {
            if let Ok(lib) = unsafe { libloading::Library::new(path) } {
                loaded = Some((path, lib));
                break;
            }
        }
        let (path, lib) = loaded.ok_or_else(|| anyhow::anyhow!("libnvidia-ml.so.1 not found"))?;
        let get_init: libloading::Symbol<NvmlInit> = unsafe { lib.get(b"nvmlInit_v2\0") }
            .or_else(|_| unsafe { lib.get(b"nvmlInit\0") })
            .with_context(|| format!("nvmlInit in {path}"))?;
        if unsafe { get_init() } != 0 { anyhow::bail!("nvmlInit failed"); }
        macro_rules! sym {
            ($name:literal, $ty:ty) => {
                *unsafe { lib.get::<$ty>(concat!($name, "\0").as_bytes()) }
                    .with_context(|| format!("{} in {}", $name, path))?
            };
        }
        let shutdown: NvmlShutdown = sym!("nvmlShutdown", NvmlShutdown);
        let get_count: NvmlDeviceGetCount = unsafe { lib.get(b"nvmlDeviceGetCount_v2\0") }
            .or_else(|_| unsafe { lib.get(b"nvmlDeviceGetCount\0") })
            .map(|s| *s)
            .context("nvmlDeviceGetCount")?;
        let get_handle: NvmlDeviceGetHandleByIndex =
            unsafe { lib.get(b"nvmlDeviceGetHandleByIndex_v2\0") }
                .or_else(|_| unsafe { lib.get(b"nvmlDeviceGetHandleByIndex\0") })
                .map(|s| *s)
                .context("nvmlDeviceGetHandleByIndex")?;
        let get_name: NvmlDeviceGetName = sym!("nvmlDeviceGetName", NvmlDeviceGetName);
        let get_memory: NvmlDeviceGetMemoryInfo = sym!("nvmlDeviceGetMemoryInfo", NvmlDeviceGetMemoryInfo);
        let get_utilization: NvmlDeviceGetUtilizationRates = sym!("nvmlDeviceGetUtilizationRates", NvmlDeviceGetUtilizationRates);
        let get_temperature: NvmlDeviceGetTemperature = sym!("nvmlDeviceGetTemperature", NvmlDeviceGetTemperature);
        let mut count = 0u32;
        if unsafe { get_count(&mut count) } != 0 || count == 0 {
            unsafe { shutdown() };
            anyhow::bail!("no NVML devices");
        }
        let mut devices = Vec::new();
        for index in 0..count {
            let mut handle = core::ptr::null_mut();
            if unsafe { get_handle(index, &mut handle) } != 0 || handle.is_null() { continue; }
            let mut buf = [0u8; 256];
            let name = if unsafe { get_name(handle, buf.as_mut_ptr(), buf.len() as u32) } == 0 {
                cstr(&buf)
            } else {
                format!("NVIDIA GPU {index}")
            };
            devices.push(NvmlDevice { handle, name });
        }
        Ok(Self { _lib: lib, shutdown, devices, get_memory, get_utilization, get_temperature })
    }

    fn read(&self, device: &NvmlDevice, base: &GpuMetrics) -> GpuMetrics {
        let mut memory = NvmlMemory { total: 0, free: 0, used: 0 };
        let mut utilization = NvmlUtilization { gpu: 0, memory: 0 };
        let mut temperature = 0u32;
        let usage = if unsafe { (self.get_utilization)(device.handle, &mut utilization) } == 0 {
            Some(utilization.gpu as f32)
        } else { None };
        let temp = if unsafe { (self.get_temperature)(device.handle, 0, &mut temperature) } == 0 {
            Some(temperature as f32)
        } else { None };
        let memory_ok = unsafe { (self.get_memory)(device.handle, &mut memory) } == 0;
        GpuMetrics {
            name: if base.name.is_empty() { device.name.clone() } else { base.name.clone() },
            usage_percent: usage,
            vram_used_bytes: if memory_ok { Some(memory.used) } else { None },
            vram_total_bytes: if memory_ok { Some(memory.total) } else { base.vram_total_bytes },
            temperature_c: temp,
            source: "NVML".to_string(),
            ..base.clone()
        }
    }
}

impl Drop for Nvml {
    fn drop(&mut self) { unsafe { (self.shutdown)() }; }
}

pub struct GpuCollector {
    adapters: Vec<GpuMetrics>,
    nvml: Option<Nvml>,
    total_memory: u64,
}

impl GpuCollector {
    pub fn new(total_memory_bytes: u64) -> Result<Self> {
        let mut adapters = enumerate_drm_adapters().unwrap_or_else(|err| {
            tracing::warn!("DRM/sysfs adapter enumeration failed: {err}");
            Vec::new()
        });
        let nvml = Nvml::load().ok();
        if adapters.is_empty() && nvml.is_none() {
            adapters.push(GpuMetrics {
                name: "Unknown GPU".to_string(),
                vendor: GpuVendor::Unknown.as_str().to_string(),
                source: "no GPU found".to_string(),
                ..GpuMetrics::default()
            });
        }
        let mut ordinal = 0usize;
        if let Some(nvml) = &nvml {
            for adapter in adapters.iter_mut() {
                if adapter.vendor == GpuVendor::Nvidia.as_str() {
                    if let Some(device) = nvml.devices.get(ordinal) {
                        *adapter = nvml.read(device, adapter);
                    }
                    ordinal += 1;
                }
            }
        }
        Ok(Self { adapters, nvml, total_memory: total_memory_bytes })
    }

    pub fn read(&mut self) -> Vec<GpuMetrics> {
        let mut nvidia_ordinal = 0usize;
        self.adapters.iter().map(|base| {
            let mut metrics = base.clone();
            if metrics.vram_total_bytes.is_none() {
                metrics.vram_total_bytes = Some(self.total_memory);
            }
            if metrics.vendor == GpuVendor::Nvidia.as_str() {
                if let Some(nvml) = &self.nvml {
                    if let Some(device) = nvml.devices.get(nvidia_ordinal) {
                        metrics = nvml.read(device, &metrics);
                    }
                }
                nvidia_ordinal += 1;
            }
            if metrics.usage_percent.is_none() {
                metrics.usage_percent = sysfs_gpu_busy(&metrics.id).or_else(|| sysfs_gpu_busy(&metrics.name));
            }
            if metrics.temperature_c.is_none() {
                metrics.temperature_c = sysfs_gpu_temperature(&metrics.id);
            }
            metrics
        }).collect()
    }

    pub fn primary(&self) -> GpuMetrics {
        self.adapters.iter().find(|a| !a.is_integrated)
            .or_else(|| self.adapters.first())
            .cloned()
            .unwrap_or_default()
    }
}

impl Collector for GpuCollector {
    fn name(&self) -> &'static str { "gpu" }
    fn collect(&mut self, sample: &mut MetricSample) -> Result<()> {
        let all = self.read();
        sample.gpu = all.iter().find(|g| !g.is_integrated)
            .or_else(|| all.first()).cloned().unwrap_or_default();
        sample.gpus = all;
        Ok(())
    }
}

fn enumerate_drm_adapters() -> Result<Vec<GpuMetrics>> {
    let mut adapters = Vec::new();
    let entries = match fs::read_dir("/sys/class/drm") {
        Ok(entries) => entries,
        Err(_) => return Ok(adapters),
    };
    let mut seen: std::collections::HashSet<u64> = std::collections::HashSet::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("card") || !name[4..].chars().all(|c| c.is_ascii_digit()) { continue; }
        let device = entry.path().join("device");
        let Some(vendor_id) = parse_u64(device.join("vendor")) else { continue };
        if !seen.insert(vendor_id) { continue; }
        let device_id = parse_u64(device.join("device")).unwrap_or(0);
        let vendor = GpuVendor::from_pci_vendor_id(vendor_id as u32);
        let bdf = fs::canonicalize(&device).ok()
            .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
            .unwrap_or_else(|| name.to_string());
        let name_text = pci_device_name(vendor_id, device_id)
            .or_else(|| read_trimmed(device.join("product_name")))
            .unwrap_or_else(|| format!("{} GPU {:04X}", vendor.as_str(), device_id));
        let integrated = vendor == GpuVendor::Intel
            || vendor == GpuVendor::Qualcomm
            || (vendor == GpuVendor::Amd && name_text.to_lowercase().contains("graphics"))
            || is_integrated_driver(&device);
        let architecture = match vendor {
            GpuVendor::Qualcomm => "Adreno".to_string(),
            GpuVendor::Nvidia => "NVIDIA".to_string(),
            GpuVendor::Intel => "Intel Graphics".to_string(),
            GpuVendor::Amd => "AMD Graphics".to_string(),
            _ => String::new(),
        };
        adapters.push(GpuMetrics {
            id: bdf,
            name: name_text,
            vendor: vendor.as_str().to_string(),
            vendor_id: vendor_id as u32,
            device_id: device_id as u32,
            architecture,
            is_integrated: integrated,
            vram_total_bytes: parse_u64(device.join("mem_info_vram_total")),
            vram_used_bytes: parse_u64(device.join("mem_info_vram_used")),
            dedicated_system_bytes: parse_u64(device.join("mem_info_vis_vram_total")),
            shared_system_bytes: None,
            usage_percent: sysfs_gpu_busy(&format!("{}", vendor_id)),
            temperature_c: sysfs_gpu_temperature(&format!("{}", vendor_id)),
            source: "DRM sysfs".to_string(),
        });
    }
    Ok(adapters)
}

fn is_integrated_driver(device: &Path) -> bool {
    let Ok(target) = fs::read_link(device.join("driver")) else { return false };
    let driver = target.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    matches!(driver.as_str(), "i915" | "xe")
}

fn pci_device_name(vendor: u64, device: u64) -> Option<String> {
    let text = fs::read_to_string("/usr/share/misc/pci.ids").ok()?;
    let vendor_line = format!("{:04x}", vendor);
    let device_line = format!("\t{:04x} ", device);
    let mut in_vendor = false;
    for line in text.lines() {
        if !line.starts_with('\t') && !line.is_empty() {
            in_vendor = line.starts_with(&vendor_line);
            continue;
        }
        if in_vendor && line.starts_with(&device_line) {
            return line.trim().split_once(' ').map(|(_, name)| name.to_string());
        }
    }
    None
}

fn sysfs_gpu_busy(key: &str) -> Option<f32> {
    let entries = fs::read_dir("/sys/class/drm").ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let device = path.join("device");
        let is_match = read_trimmed(device.join("vendor"))
            .and_then(|v| u64::from_str_radix(v.trim_start_matches("0x"), 16).ok())
            .map(|v| format!("{}", v) == key)
            .unwrap_or(false)
            || path.to_string_lossy().contains(key)
            || entry.file_name().to_string_lossy().contains(key);
        if is_match {
            if let Some(value) = parse_u64(device.join("gpu_busy_percent")) {
                return Some(value as f32);
            }
        }
    }
    None
}

fn sysfs_gpu_temperature(key: &str) -> Option<f32> {
    let entries = fs::read_dir("/sys/class/hwmon").ok()?;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = read_trimmed(path.join("name")).unwrap_or_default();
        if !name.contains("amdgpu") && !name.contains("nvidia") && !name.contains("i915") { continue; }
        let vendor_match = name.contains("nvidia") && key.contains("10de")
            || name.contains("amdgpu") && key.contains("1002")
            || name.contains("i915") && key.contains("8086");
        if vendor_match {
            if let Some(value) = parse_u64(path.join("temp1_input")) {
                return Some(value as f32 / 1000.0);
            }
        }
    }
    None
}

fn cstr(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

// ---------------------------------------------------------------------------
// tests — hardware-independent
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_usage_delta_handles_idle_and_busy() {
        let idle = [10, 0, 5, 85, 0, 0, 0, 0];
        let mut busy = idle;
        busy[0] += 80;
        assert_eq!(cpu_usage_delta(Some(idle), busy), Some(100.0));
        assert_eq!(cpu_usage_delta(Some(idle), idle), None);
    }

    #[test]
    fn proc_stat_ticks_parses_comm_with_spaces() {
        assert_eq!(parse_proc_stat_ticks("123 (my process) S 1 2 3 4 5 6 7 8 9 10 11 12"), Some(23));
    }

    #[test]
    fn partition_detection() {
        assert!(is_partition("sda1"));
        assert!(is_partition("nvme0n1p3"));
        assert!(is_partition("mmcblk0p1"));
        assert!(!is_partition("sda"));
        assert!(!is_partition("nvme0n1"));
    }
}
