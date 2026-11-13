use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SystemInfo {
    pub hostname: String,
    pub os_version: String,
    pub build_number: u32,
    pub username: String,
    pub logical_cores: usize,
    pub total_memory_bytes: u64,
    pub boot_time_ms: Option<u64>,
}
