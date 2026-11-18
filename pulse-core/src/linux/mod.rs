pub mod desktop;

#[cfg(target_os = "windows")]
pub use crate::windows::startup;
#[cfg(target_os = "linux")]
pub use desktop as startup;
