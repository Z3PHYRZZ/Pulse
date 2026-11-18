// language: Rust, file: lib.rs, target: Windows 11 x86_64, edition 2021
pub mod collectors;
pub mod config;
pub mod engine;
pub mod models;
pub mod utils;
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub use windows::startup;
#[cfg(target_os = "linux")]
pub use linux::startup;

pub use config::TrayEvent;
pub use config::{Config, ConfigState, ThemeMode};
pub use engine::EngineHandle;
pub use models::{MetricSample, SystemInfo};
#[cfg(target_os = "windows")]
pub use windows::tray::{force_exit, Tray};
#[cfg(target_os = "linux")]
pub use linux::desktop::{force_exit, Tray};
