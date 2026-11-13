use crate::models::{MetricSample, ProcessInfo};
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::mem::{size_of, zeroed};
use std::time::Instant;
use windows::Win32::Foundation::{CloseHandle, FILETIME, HANDLE};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
use windows::Win32::System::Threading::{
    GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};

pub struct ProcessCollector {
    previous: HashMap<u32, u64>,
    previous_at: Instant,
    cpu_count: usize,
}

impl ProcessCollector {
    pub fn new(cpu_count: usize) -> Self {
        Self {
            previous: HashMap::new(),
            previous_at: Instant::now(),
            cpu_count: cpu_count.max(1),
        }
    }

    pub fn read(&mut self) -> Result<Vec<ProcessInfo>> {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) }
            .context("CreateToolhelp32Snapshot")?;
        let now = Instant::now();
        let elapsed = now
            .duration_since(self.previous_at)
            .as_secs_f64()
            .max(0.001);
        let mut entry: PROCESSENTRY32W = unsafe { zeroed() };
        entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;
        let mut processes = Vec::new();
        let mut current = HashMap::new();
        let mut ok = unsafe { Process32FirstW(snapshot, &mut entry) }.is_ok();
        while ok {
            let pid = entry.th32ProcessID;
            let name = wide_to_string(&entry.szExeFile);
            let (cpu_ticks, memory) = read_process_details(pid);
            let cpu_percent = if let Some(previous) = self.previous.get(&pid) {
                let delta = cpu_ticks.saturating_sub(*previous);
                let seconds = delta as f64 / 10_000_000.0;
                ((seconds / elapsed) * 100.0 / self.cpu_count as f64) as f32
            } else {
                0.0
            };
            current.insert(pid, cpu_ticks);
            processes.push(ProcessInfo {
                pid,
                name,
                cpu_percent: cpu_percent.clamp(0.0, 100.0),
                memory_bytes: memory,
            });
            ok = unsafe { Process32NextW(snapshot, &mut entry) }.is_ok();
        }
        unsafe { CloseHandle(snapshot) }.ok();
        self.previous = current;
        self.previous_at = now;
        Ok(processes)
    }
}

impl crate::collectors::Collector for ProcessCollector {
    fn name(&self) -> &'static str {
        "processes"
    }

    fn collect(&mut self, sample: &mut MetricSample) -> Result<()> {
        sample.processes = self.read()?;
        Ok(())
    }
}

fn read_process_details(pid: u32) -> (u64, u64) {
    let handle: HANDLE = match unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }
    {
        Ok(handle) => handle,
        Err(_) => return (0, 0),
    };
    let mut creation: FILETIME = unsafe { zeroed() };
    let mut exit: FILETIME = unsafe { zeroed() };
    let mut kernel: FILETIME = unsafe { zeroed() };
    let mut user: FILETIME = unsafe { zeroed() };
    let times =
        unsafe { GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) };
    let ticks = if times.is_ok() {
        filetime_to_u64(kernel).saturating_add(filetime_to_u64(user))
    } else {
        0
    };
    let mut counters: PROCESS_MEMORY_COUNTERS = unsafe { zeroed() };
    counters.cb = size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
    let memory = if unsafe { GetProcessMemoryInfo(handle, &mut counters, counters.cb) }.is_ok() {
        counters.WorkingSetSize as u64
    } else {
        0
    };
    unsafe { CloseHandle(handle) }.ok();
    (ticks, memory)
}

fn filetime_to_u64(value: FILETIME) -> u64 {
    ((value.dwHighDateTime as u64) << 32) | value.dwLowDateTime as u64
}

fn wide_to_string(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}
