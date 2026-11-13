// language: Rust, file: lib.rs, target: Windows 11 x86_64, edition 2021
pub mod collectors;
pub mod config;
pub mod engine;
pub mod models;
pub mod utils;
pub mod windows;

pub use config::TrayEvent;
pub use config::{Config, ConfigState, ThemeMode};
pub use engine::EngineHandle;
pub use models::{MetricSample, SystemInfo};
pub use windows::tray::{force_exit, Tray};
