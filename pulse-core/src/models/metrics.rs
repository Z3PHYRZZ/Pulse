use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GpuVendor {
    Nvidia,
    Intel,
    Amd,
    Qualcomm,
    Microsoft,
    #[default]
    Unknown,
}

impl GpuVendor {
    pub fn from_pci_vendor_id(id: u32) -> Self {
        match id {
            0x10DE => Self::Nvidia,
            0x8086 => Self::Intel,
            0x1002 | 0x1022 => Self::Amd,
            0x5143 => Self::Qualcomm,
            0x1414 => Self::Microsoft,
            _ => Self::Unknown,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Nvidia => "NVIDIA",
            Self::Intel => "Intel",
            Self::Amd => "AMD",
            Self::Qualcomm => "Qualcomm",
            Self::Microsoft => "Microsoft",
            Self::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuMetrics {
    pub usage_percent: f32,
    pub per_core_percent: Vec<f32>,
    pub frequency_mhz: Option<f32>,
    pub logical_cores: usize,
    pub name: String,
    pub temperature_c: Option<f32>,
    pub architecture: String,
    pub is_arm64: bool,
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
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub vendor_id: u32,
    pub device_id: u32,
    pub architecture: String,
    pub is_integrated: bool,
    pub usage_percent: Option<f32>,
    pub vram_used_bytes: Option<u64>,
    pub vram_total_bytes: Option<u64>,
    pub dedicated_system_bytes: Option<u64>,
    pub shared_system_bytes: Option<u64>,
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
    pub gpus: Vec<GpuMetrics>,
    pub disk: DiskMetrics,
    pub network: NetworkMetrics,
    pub processes: Vec<ProcessInfo>,
    pub uptime_seconds: u64,
    pub warnings: Vec<String>,
}

use crate::models::process::ProcessInfo;

#[cfg(test)]
mod tests {
    use super::GpuVendor;

    #[test]
    fn vendor_ids_map_all_supported_gpu_vendors() {
        assert_eq!(GpuVendor::from_pci_vendor_id(0x10DE), GpuVendor::Nvidia);
        assert_eq!(GpuVendor::from_pci_vendor_id(0x8086), GpuVendor::Intel);
        assert_eq!(GpuVendor::from_pci_vendor_id(0x1002), GpuVendor::Amd);
        assert_eq!(GpuVendor::from_pci_vendor_id(0x1022), GpuVendor::Amd);
        assert_eq!(GpuVendor::from_pci_vendor_id(0x5143), GpuVendor::Qualcomm);
        assert_eq!(GpuVendor::from_pci_vendor_id(0x1414), GpuVendor::Microsoft);
        assert_eq!(GpuVendor::from_pci_vendor_id(0xDEAD), GpuVendor::Unknown);
    }

    #[test]
    fn vendor_labels_are_stable() {
        assert_eq!(GpuVendor::Nvidia.as_str(), "NVIDIA");
        assert_eq!(GpuVendor::Intel.as_str(), "Intel");
        assert_eq!(GpuVendor::Amd.as_str(), "AMD");
        assert_eq!(GpuVendor::Qualcomm.as_str(), "Qualcomm");
    }
}
