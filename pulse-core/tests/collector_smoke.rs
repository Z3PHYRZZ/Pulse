// language: Rust, file: collector_smoke.rs, target: Windows 11 x86_64
#![cfg(windows)]

use std::mem::{size_of, zeroed};
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::System::Performance::{
    PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterValue,
    PdhOpenQueryW, PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY,
};
use windows::Win32::System::SystemInformation::{
    ComputerNameDnsHostname, GetComputerNameExW, GetSystemInfo, GetTickCount64,
    GlobalMemoryStatusEx, MEMORYSTATUSEX, SYSTEM_INFO,
};

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(std::iter::once(0)).collect()
}

fn pdh_counter_ready(_query: PDH_HQUERY, counter: PDH_HCOUNTER) -> bool {
    let mut value: PDH_FMT_COUNTERVALUE = unsafe { zeroed() };
    let status = unsafe { PdhGetFormattedCounterValue(counter, PDH_FMT_DOUBLE, None, &mut value) };
    status == 0 && value.CStatus == 0 && unsafe { value.Anonymous.doubleValue }.is_finite()
}

#[test]
fn pdh_cpu_counter_reports_a_finite_double() {
    let mut query = PDH_HQUERY::default();
    assert_eq!(unsafe { PdhOpenQueryW(PCWSTR::null(), 0, &mut query) }, 0);
    let paths = [
        "\\Processor(_Total)\\% Processor Time",
        "\\Processor Information(_Total)\\% Processor Utility",
    ];
    let mut counter = PDH_HCOUNTER::default();
    let mut added = false;
    for path in paths {
        let wide_path = wide(path);
        if unsafe {
            PdhAddEnglishCounterW(query, PCWSTR::from_raw(wide_path.as_ptr()), 0, &mut counter)
        } == 0
        {
            added = true;
            break;
        }
        counter = PDH_HCOUNTER::default();
    }
    assert!(added, "no English CPU counter path was accepted");
    let mut ready = false;
    for _ in 0..5 {
        std::thread::sleep(std::time::Duration::from_millis(1100));
        assert_eq!(unsafe { PdhCollectQueryData(query) }, 0);
        if pdh_counter_ready(query, counter) {
            ready = true;
            break;
        }
    }
    assert!(ready, "CPU counter never produced a valid sample");
    unsafe { PdhCloseQuery(query) };
}

#[test]
fn global_memory_status_reports_physical_memory() {
    let mut status: MEMORYSTATUSEX = unsafe { zeroed() };
    status.dwLength = size_of::<MEMORYSTATUSEX>() as u32;
    unsafe { GlobalMemoryStatusEx(&mut status) }.expect("GlobalMemoryStatusEx");
    assert!(status.ullTotalPhys > 0);
    assert!(status.ullAvailPhys <= status.ullTotalPhys);
}

#[test]
fn system_information_reports_cores_and_uptime() {
    let mut info: SYSTEM_INFO = unsafe { zeroed() };
    unsafe { GetSystemInfo(&mut info) };
    assert!(info.dwNumberOfProcessors > 0);
    assert!(unsafe { GetTickCount64() } > 0);
    let mut size = 0u32;
    let _ = unsafe { GetComputerNameExW(ComputerNameDnsHostname, None, &mut size) };
    assert!(size > 0);
    let mut buf = vec![0u16; size as usize + 1];
    unsafe {
        GetComputerNameExW(
            ComputerNameDnsHostname,
            Some(PWSTR(buf.as_mut_ptr())),
            &mut size,
        )
    }
    .expect("GetComputerNameExW");
    let hostname = String::from_utf16_lossy(&buf[..size as usize]);
    assert!(!hostname.is_empty());
}
