# Changelog

All notable changes to Pulse are documented here. Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.1.0] — 2026-11-13

### Added
- Dark and Light mode buttons in the sidebar and Settings page.
- Persisted `theme` value in `config.toml`; theme selection applies immediately to the Iced window.
- Native `WM_CLOSE` hook on the main window thread so the title-bar close button terminates the full process.
- Responsive monitoring-engine shutdown through a stop-aware scheduler wait loop.
- Explicit `Hide to tray` path for background operation; close no longer leaves the process running.
- `CHANGELOG.md` and GitHub Release download links for portable and installer builds.

### Changed
- Raised package, lockfile, installer, README, and default configuration metadata to 1.1.0.
- Reworked close handling: `exit_on_close_request` remains enabled and no close-request subscription minimizes the window.
- Sidebar and tray `Exit` actions route through `ExitProcess(0)` after tray shutdown.
- README, roadmap, Windows API notes, and Settings page now state that the close button always exits fully.

### Fixed
- Close button now fully exits both visible windows and `--minimized` startup windows instead of leaving Pulse running in the background.
- Dark/Light button state now follows the persisted configuration instead of a static dark-only theme.
- Engine shutdown no longer waits for the entire configured refresh interval.

### Downloads
- `Pulse-1.1.0-setup-win-x64.exe` — Inno Setup installer for Windows 10/11 x64.
- `Pulse-1.1.0-portable-win-x64.exe` — standalone executable; run from any writable folder.

## [1.0.0] — 2026-11-13

### Added
- Native Windows system monitor built with Rust 2021, Iced 0.13, and `windows-rs`.
- Dashboard with CPU, memory, GPU, storage, network, process, system, and settings pages.
- Collector abstraction backed by PDH, `GlobalMemoryStatusEx`, Tool Help, DXGI, dynamic NVML loading, `GetDiskFreeSpaceExW`, and registry reads.
- Iced Canvas sparklines with bounded metric history and configurable refresh cadence.
- Process table with CPU/RAM sorting, filtering, and configurable row cap.
- TOML configuration at `%APPDATA%\Pulse\config.toml`.
- Native system tray, Hide/Show tray actions, and optional Windows startup integration.
- Release profile, Inno Setup installer script, MIT license, README, architecture notes, Windows API notes, and roadmap.
