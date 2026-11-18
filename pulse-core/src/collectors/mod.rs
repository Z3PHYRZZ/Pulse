//! Platform-specific collector modules.
//!
//! The aggregator depends only on `platform::{CpuCollector, ...}` and the
//! `Collector` trait. Each OS supplies its own implementations behind the
//! facade in `platform.rs`, so the engine never sees a Win32 or /proc type.

use crate::models::MetricSample;
use anyhow::Result;

pub type CollectResult<T> = Result<T>;

pub trait Collector: Send {
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    fn collect(&mut self, sample: &mut MetricSample) -> CollectResult<()>;
}

#[cfg(target_os = "windows")]
pub mod platform {
    pub use super::windows_impl::{
        CpuCollector, DiskCollector, GpuCollector, MemoryCollector, NetworkCollector,
        ProcessCollector,
    };
    pub use super::system::apply_uptime;
    pub use super::system::ensure;
    pub use super::system::read_system_info;
}

#[cfg(target_os = "linux")]
pub mod platform {
    pub use super::linux_impl::{
        CpuCollector, DiskCollector, GpuCollector, MemoryCollector, NetworkCollector,
        ProcessCollector,
    };
    pub use super::linux_impl::{apply_uptime, ensure, read_system_info};
}

#[cfg(target_os = "windows")]
mod windows_impl {
    pub use super::cpu::CpuCollector;
    pub use super::disk::DiskCollector;
    pub use super::gpu::GpuCollector;
    pub use super::memory::MemoryCollector;
    pub use super::network::NetworkCollector;
    pub use super::processes::ProcessCollector;
}

#[cfg(target_os = "windows")]
pub mod cpu;
#[cfg(target_os = "windows")]
pub mod disk;
#[cfg(target_os = "windows")]
pub mod gpu;
#[cfg(target_os = "windows")]
pub mod memory;
#[cfg(target_os = "windows")]
pub mod network;
#[cfg(target_os = "windows")]
pub mod pdh;
#[cfg(target_os = "windows")]
pub mod processes;

#[cfg(target_os = "linux")]
pub mod linux_impl;

#[cfg(target_os = "windows")]
pub mod system;

#[cfg(not(any(target_os = "windows", target_os = "linux")))]
compile_error!("Pulse supports Windows and Linux only");
