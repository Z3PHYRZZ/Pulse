use crate::collectors::pdh::PdhQuery;
use crate::models::{DiskMetrics, MetricSample, VolumeUsage};
use anyhow::{Context, Result};
use windows::core::PCWSTR;
use windows::Win32::Storage::FileSystem::{GetDiskFreeSpaceExW, GetLogicalDriveStringsW};

const DISK_READ: &str = "\\LogicalDisk(_Total)\\Disk Read Bytes/sec";
const DISK_WRITE: &str = "\\LogicalDisk(_Total)\\Disk Write Bytes/sec";

pub struct DiskCollector {
    query: PdhQuery,
    read: Option<usize>,
    write: Option<usize>,
}

impl DiskCollector {
    pub fn new() -> Result<Self> {
        let mut query = PdhQuery::new()?;
        let read = query.add_counter(DISK_READ).ok();
        let write = query.add_counter(DISK_WRITE).ok();
        query.warm_up();
        Ok(Self { query, read, write })
    }

    pub fn read(&mut self) -> Result<DiskMetrics> {
        self.query.collect().context("PDH disk collect")?;
        Ok(DiskMetrics {
            read_bytes_per_sec: self
                .read
                .and_then(|i| self.query.double(i))
                .unwrap_or(0.0)
                .max(0.0),
            write_bytes_per_sec: self
                .write
                .and_then(|i| self.query.double(i))
                .unwrap_or(0.0)
                .max(0.0),
            volumes: enumerate_volumes(),
        })
    }
}

impl crate::collectors::Collector for DiskCollector {
    fn name(&self) -> &'static str {
        "disk"
    }

    fn collect(&mut self, sample: &mut MetricSample) -> Result<()> {
        sample.disk = self.read()?;
        Ok(())
    }
}

fn enumerate_volumes() -> Vec<VolumeUsage> {
    let mut size = unsafe { GetLogicalDriveStringsW(None) };
    if size == 0 {
        return Vec::new();
    }
    size += 1;
    let mut buffer = vec![0u16; size as usize];
    let copied = unsafe { GetLogicalDriveStringsW(Some(buffer.as_mut_slice())) };
    if copied == 0 {
        return Vec::new();
    }
    let mut out = Vec::new();
    let mut start = 0usize;
    for (i, &ch) in buffer.iter().enumerate() {
        if ch != 0 {
            continue;
        }
        if i == start {
            start = i + 1;
            continue;
        }
        let name = String::from_utf16_lossy(&buffer[start..i]);
        let mut wide: Vec<u16> = name.encode_utf16().collect();
        wide.push(0);
        let mut free: u64 = 0;
        let mut total: u64 = 0;
        let result = unsafe {
            GetDiskFreeSpaceExW(
                PCWSTR::from_raw(wide.as_ptr()),
                None,
                Some(&mut total),
                Some(&mut free),
            )
        };
        if result.is_ok() && total > 0 {
            let used = total.saturating_sub(free);
            out.push(VolumeUsage {
                name,
                used_bytes: used,
                total_bytes: total,
                usage_percent: used as f32 / total as f32 * 100.0,
            });
        }
        start = i + 1;
        if start >= buffer.len() {
            break;
        }
    }
    out
}
