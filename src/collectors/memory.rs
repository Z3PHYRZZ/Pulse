use crate::models::{MemoryMetrics, MetricSample};
use anyhow::{Context, Result};
use std::mem::{size_of, zeroed};
use windows::Win32::System::SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX};

#[derive(Default)]
pub struct MemoryCollector;

impl MemoryCollector {
    pub fn new() -> Self {
        Self
    }

    pub fn read(&mut self) -> Result<MemoryMetrics> {
        let mut status: MEMORYSTATUSEX = unsafe { zeroed() };
        status.dwLength = size_of::<MEMORYSTATUSEX>() as u32;
        unsafe { GlobalMemoryStatusEx(&mut status) }.context("GlobalMemoryStatusEx")?;
        let used = status.ullTotalPhys.saturating_sub(status.ullAvailPhys);
        let percent = if status.ullTotalPhys == 0 {
            0.0
        } else {
            used as f64 / status.ullTotalPhys as f64 * 100.0
        };
        Ok(MemoryMetrics {
            used_bytes: used,
            available_bytes: status.ullAvailPhys,
            total_bytes: status.ullTotalPhys,
            usage_percent: percent as f32,
        })
    }
}

impl crate::collectors::Collector for MemoryCollector {
    fn name(&self) -> &'static str {
        "memory"
    }

    fn collect(&mut self, sample: &mut MetricSample) -> Result<()> {
        sample.memory = self.read()?;
        Ok(())
    }
}
