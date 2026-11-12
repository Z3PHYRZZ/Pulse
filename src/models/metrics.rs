use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage_percent: f32,
    pub per_core_percent: Vec<f32>,
    pub frequency_mhz: Option<f32>,
    pub logical_cores: usize,
    pub name: String,
    pub temperature_c: Option<f32>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MemoryMetrics {
    pub used_bytes: u64,
    pub available_bytes: u64,
    pub total_bytes: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GpuMetrics {
    pub name: String,
    pub usage_percent: Option<f32>,
    pub vram_used_bytes: Option<u64>,
    pub vram_total_bytes: Option<u64>,
    pub temperature_c: Option<f32>,
    pub source: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiskMetrics {
    pub read_bytes_per_sec: f64,
    pub write_bytes_per_sec: f64,
    pub volumes: Vec<VolumeUsage>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VolumeUsage {
    pub name: String,
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub usage_percent: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub upload_bytes_per_sec: f64,
    pub download_bytes_per_sec: f64,
    pub adapters: Vec<AdapterTraffic>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdapterTraffic {
    pub name: String,
    pub upload_bytes_per_sec: f64,
    pub download_bytes_per_sec: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricSample {
    pub timestamp_ms: u64,
    pub cpu: CpuMetrics,
    pub memory: MemoryMetrics,
    pub gpu: GpuMetrics,
    pub disk: DiskMetrics,
    pub network: NetworkMetrics,
    pub processes: Vec<ProcessInfo>,
    pub uptime_seconds: u64,
    pub warnings: Vec<String>,
}

use crate::models::process::ProcessInfo;
