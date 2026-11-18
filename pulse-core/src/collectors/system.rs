use crate::models::MetricSample;
use crate::models::SystemInfo;
use anyhow::{anyhow, Context, Result};
use std::mem::{size_of, zeroed};
use windows::core::PCWSTR;
use windows::Win32::Foundation::GetLastError;
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ,
};
use windows::Win32::System::SystemInformation::{
    ComputerNameDnsHostname, GetComputerNameExW, GetNativeSystemInfo, GetTickCount64,
    GlobalMemoryStatusEx, MEMORYSTATUSEX, PROCESSOR_ARCHITECTURE_AMD64,
    PROCESSOR_ARCHITECTURE_ARM32_ON_WIN64, PROCESSOR_ARCHITECTURE_ARM64,
    PROCESSOR_ARCHITECTURE_IA32_ON_ARM64, PROCESSOR_ARCHITECTURE_INTEL, SYSTEM_INFO,
};

pub struct SystemCollector {
    info: SystemInfo,
}

impl SystemCollector {
    pub fn new() -> Result<Self> {
        Ok(Self {
            info: read_system_info()?,
        })
    }

    pub fn info(&self) -> &SystemInfo {
        &self.info
    }
}

pub fn read_system_info() -> Result<SystemInfo> {
    let hostname = read_computer_name()?;
    let os_version = read_os_version().unwrap_or_else(|_| "Windows".to_string());
    let build_number = read_build_number().unwrap_or(0);
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "unknown".to_string());
    let mut sysinfo: SYSTEM_INFO = unsafe { zeroed() };
    // GetNativeSystemInfo reports the true process architecture; GetSystemInfo
    // can report an emulated WOW64 view on x64 hosts.
    unsafe { GetNativeSystemInfo(&mut sysinfo) };
    let mut mem: MEMORYSTATUSEX = unsafe { zeroed() };
    mem.dwLength = size_of::<MEMORYSTATUSEX>() as u32;
    unsafe { GlobalMemoryStatusEx(&mut mem) }.context("GlobalMemoryStatusEx")?;
    let uptime_ms = unsafe { GetTickCount64() };
    let boot_time_ms = crate::utils::platform::now_ms().checked_sub(uptime_ms);
    let arch_value = unsafe { sysinfo.Anonymous.Anonymous.wProcessorArchitecture }.0;
    let (processor_architecture, is_arm64) = architecture_label(arch_value);
    let cpu_name = read_cpu_name().unwrap_or_default();
    Ok(SystemInfo {
        hostname,
        os_version,
        build_number,
        username,
        logical_cores: sysinfo.dwNumberOfProcessors as usize,
        total_memory_bytes: mem.ullTotalPhys,
        boot_time_ms,
        processor_architecture,
        is_arm64,
        cpu_name,
    })
}

fn architecture_label(arch: u16) -> (String, bool) {
    match arch {
        a if a == PROCESSOR_ARCHITECTURE_ARM64.0 => ("ARM64".to_string(), true),
        a if a == PROCESSOR_ARCHITECTURE_ARM32_ON_WIN64.0 => ("ARM32-on-ARM64".to_string(), true),
        a if a == PROCESSOR_ARCHITECTURE_IA32_ON_ARM64.0 => ("x86-on-ARM64".to_string(), true),
        a if a == PROCESSOR_ARCHITECTURE_AMD64.0 => ("x64".to_string(), false),
        a if a == PROCESSOR_ARCHITECTURE_INTEL.0 => ("x86".to_string(), false),
        other => (format!("0x{other:04X}"), false),
    }
}

fn read_cpu_name() -> Result<String> {
    const KEY: &str = "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0";
    const VALUE: &str = "ProcessorNameString";
    let key = open_registry_key(KEY)?;
    let value = read_registry_string(key, VALUE)?;
    let _ = unsafe { RegCloseKey(key) };
    Ok(value)
}

fn read_computer_name() -> Result<String> {
    let mut size = 0u32;
    let _ = unsafe { GetComputerNameExW(ComputerNameDnsHostname, None, &mut size) };
    if size == 0 {
        return Err(anyhow!("GetComputerNameExW returned zero size"));
    }
    let mut buf = vec![0u16; size as usize + 1];
    unsafe {
        GetComputerNameExW(
            ComputerNameDnsHostname,
            Some(windows::core::PWSTR(buf.as_mut_ptr())),
            &mut size,
        )
        .context("GetComputerNameExW")?;
    }
    Ok(wide_to_string(&buf, size as usize))
}

fn read_os_version() -> Result<String> {
    const PRODUCT_NAME: &str = "Software\\Microsoft\\Windows NT\\CurrentVersion";
    const PRODUCT: &str = "ProductName";
    let key = open_registry_key(PRODUCT_NAME)?;
    let value = read_registry_string(key, PRODUCT)?;
    let _ = unsafe { RegCloseKey(key) };
    Ok(value)
}

fn read_build_number() -> Result<u32> {
    const CURRENT: &str = "Software\\Microsoft\\Windows NT\\CurrentVersion";
    const BUILD: &str = "CurrentBuild";
    let key = open_registry_key(CURRENT)?;
    let value = read_registry_string(key, BUILD)?;
    let _ = unsafe { RegCloseKey(key) };
    value.parse::<u32>().map_err(Into::into)
}

fn open_registry_key(subkey: &str) -> Result<HKEY> {
    let mut wide: Vec<u16> = subkey.encode_utf16().collect();
    wide.push(0);
    let mut key = HKEY::default();
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR::from_raw(wide.as_ptr()),
            None,
            KEY_READ,
            &mut key,
        )
    };
    if status.is_err() {
        return Err(anyhow!("RegOpenKeyExW failed: {:?}", status));
    }
    Ok(key)
}

fn read_registry_string(key: HKEY, name: &str) -> Result<String> {
    let mut wide: Vec<u16> = name.encode_utf16().collect();
    wide.push(0);
    let mut value_type = REG_SZ;
    let mut size = 0u32;
    let status = unsafe {
        RegQueryValueExW(
            key,
            PCWSTR::from_raw(wide.as_ptr()),
            None,
            Some(&mut value_type),
            None,
            Some(&mut size),
        )
    };
    if status.is_err() {
        return Err(anyhow!("RegQueryValueExW size failed: {:?}", status));
    }
    let mut bytes = vec![0u8; size as usize];
    let status = unsafe {
        RegQueryValueExW(
            key,
            PCWSTR::from_raw(wide.as_ptr()),
            None,
            Some(&mut value_type),
            Some(bytes.as_mut_ptr()),
            Some(&mut size),
        )
    };
    if status.is_err() {
        return Err(anyhow!("RegQueryValueExW read failed: {:?}", status));
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    Ok(wide_to_string(&units, units.len()))
}

fn wide_to_string(buf: &[u16], len: usize) -> String {
    let end = buf[..len.min(buf.len())]
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(len.min(buf.len()));
    String::from_utf16_lossy(&buf[..end])
}

pub fn apply_uptime(sample: &mut MetricSample) {
    sample.uptime_seconds = unsafe { GetTickCount64() } / 1000;
}

pub fn ensure(system: &SystemInfo, sample: &mut MetricSample) {
    sample.cpu.logical_cores = system.logical_cores;
    if sample.cpu.name.is_empty() {
        let name = if system.cpu_name.is_empty() {
            format!("{} ({} logical cores)", system.os_version, system.logical_cores)
        } else {
            system.cpu_name.clone()
        };
        sample.cpu.name = name;
    }
    sample.cpu.architecture = system.processor_architecture.clone();
    sample.cpu.is_arm64 = system.is_arm64;
}

#[allow(dead_code)]
pub fn last_error() -> u32 {
    unsafe { GetLastError().0 }
}
