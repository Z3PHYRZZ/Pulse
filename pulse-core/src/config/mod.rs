use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::Duration;

const DEFAULT_TOML: &str = include_str!("../../config.default.toml");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    Show,
    Hide,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

impl ThemeMode {
    pub fn label(self) -> &'static str {
        match self {
            ThemeMode::Dark => "Dark",
            ThemeMode::Light => "Light",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            ThemeMode::Dark => "dark",
            ThemeMode::Light => "light",
        }
    }

    pub fn parse(value: &str) -> Self {
        if value.eq_ignore_ascii_case("light") {
            ThemeMode::Light
        } else {
            ThemeMode::Dark
        }
    }

    pub fn is_light(self) -> bool {
        matches!(self, ThemeMode::Light)
    }
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub refresh_ms: u64,
    pub history_seconds: usize,
    pub max_process_rows: usize,
    pub process_sort: String,
    pub start_minimized: bool,
    pub minimize_to_tray: bool,
    pub enable_startup: bool,
    pub theme: String,
}

impl Default for Config {
    fn default() -> Self {
        toml::from_str(DEFAULT_TOML).expect("config.default.toml must parse")
    }
}

impl Config {
    pub fn refresh_interval(&self) -> Duration {
        Duration::from_millis(self.refresh_ms.clamp(250, 10_000))
    }

    pub fn history_len(&self) -> usize {
        self.history_seconds.clamp(10, 600)
    }

    pub fn theme_mode(&self) -> ThemeMode {
        ThemeMode::parse(&self.theme)
    }

    pub fn sort_key(&self) -> crate::models::ProcessSort {
        match self.process_sort.to_ascii_lowercase().as_str() {
            "memory" | "ram" => crate::models::ProcessSort::Memory,
            "name" => crate::models::ProcessSort::Name,
            "pid" => crate::models::ProcessSort::Pid,
            _ => crate::models::ProcessSort::Cpu,
        }
    }

    pub fn path() -> Result<PathBuf> {
        let base = dirs::config_dir().context("no user config directory")?;
        Ok(base.join("Pulse").join("config.toml"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        let text = std::fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))?;
        let config =
            toml::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
        Ok(config)
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, toml::to_string_pretty(self)?)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct ConfigState {
    inner: RwLock<Config>,
    path: PathBuf,
}

impl ConfigState {
    pub fn load() -> Result<Self> {
        Ok(Self {
            inner: RwLock::new(Config::load()?),
            path: Config::path()?,
        })
    }

    pub fn get(&self) -> Config {
        self.inner.read().expect("config lock poisoned").clone()
    }

    pub fn set(&self, next: Config) -> Result<()> {
        next.save(&self.path)?;
        *self.inner.write().expect("config lock poisoned") = next;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_mode_round_trips() {
        assert_eq!(ThemeMode::parse("dark"), ThemeMode::Dark);
        assert_eq!(ThemeMode::parse("LIGHT"), ThemeMode::Light);
        assert_eq!(ThemeMode::Dark.as_str(), "dark");
        assert_eq!(ThemeMode::Light.to_string(), "light");
    }

    #[test]
    fn default_config_parses() {
        let config = Config::default();
        assert_eq!(config.theme_mode(), ThemeMode::Dark);
        assert_eq!(config.refresh_interval(), Duration::from_secs(1));
        assert_eq!(config.history_len(), 180);
    }
}
