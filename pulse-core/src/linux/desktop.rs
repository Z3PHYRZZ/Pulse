// language: Rust, file: linux/desktop.rs, target: Linux, edition 2021
//! Linux desktop integration: XDG autostart and API parity with the Windows
//! shell helpers. The Tauri application owns its own status-item tray on
//! Linux, so the legacy Win32 tray seam is only a no-op here.

use crate::config::TrayEvent;
use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

const DESKTOP_FILE: &str = "com.zephyrzz.pulse.desktop";

fn autostart_path() -> Result<PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(".config")))
        .context("no user config directory")?;
    Ok(base.join("autostart").join(DESKTOP_FILE))
}

fn desktop_entry() -> Result<String> {
    let exe = std::env::current_exe().context("current exe")?;
    Ok(format!(
        "[Desktop Entry]\nType=Application\nName=Pulse\nComment=Native system monitor\nExec={} --minimized\nTerminal=false\nCategories=Utility;Monitor;\nX-GNOME-Autostart-enabled=true\n",
        exe.display()
    ))
}

pub fn is_enabled() -> bool {
    autostart_path().map(|path| path.exists()).unwrap_or(false)
}

pub fn enable() -> Result<()> {
    let path = autostart_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
    }
    fs::write(&path, desktop_entry()?).with_context(|| format!("writing {}", path.display()))?;
    Ok(())
}

pub fn disable() -> Result<()> {
    let path = autostart_path()?;
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("removing {}", path.display()))?;
    }
    Ok(())
}

/// The Tauri shell owns the Linux status item. This keeps the pulse-core API
/// identical across platforms for legacy callers.
pub struct Tray {
    _events: Sender<TrayEvent>,
}

impl Tray {
    pub fn spawn(_events: Sender<TrayEvent>) -> Result<Self> {
        Ok(Self { _events: _events })
    }
}

pub fn force_exit() {
    // The Tauri window close handler performs the full shutdown on Linux.
}
