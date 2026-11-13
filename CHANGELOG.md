# Changelog
All notable changes to Pulse are documented here. Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.3] — 2026-11-13
### Fixed
- Dropped the archived `legacy-iced` (Iced 0.13) crate from the workspace so CI no longer builds a dead dependency tree.
- Removed the duplicate native Win32 tray and its polling thread; one system tray is now owned by Tauri's `TrayIconBuilder`.
- Reworked the `WM_CLOSE` hook to keep one original window procedure per HWND, dropping the shared `thread_local[4]` index and the `0x8000` message collision.
- Replaced the `ExitProcess` force-exit with a graceful `prevent_close()` + `app.exit(0)` on close.
- Re-homed the orphaned root collector smoke tests into `pulse-core/tests/` so `cargo test --workspace` actually runs them (3/3 pass).
- Rewrote `docs/ARCHITECTURE.md` and `docs/WINDOWS_API.md` to describe the live Tauri 2 layout instead of the removed Iced GUI and native tray.

### Build
- Scoped CI check/test to `--workspace` and the release build to `-p pulse`.
- `cargo check`, `cargo test` (5 tests), and `cargo build --release -p pulse` all pass.

### Downloads
- `Pulse-2.0.3-portable-win-x64.exe` — standalone Tauri executable.
- The Inno Setup installer is not included in this release; it cannot be rebuilt in this environment.


## [2.0.2] — 2026-11-13
### Fixed
- Corrected the native `WM_CLOSE` hook so each hooked window uses its own thread-local original window procedure instead of a shared global index.
- Guarded title-bar close handling against stale window-procedure chains on Tauri's webview windows.
- Kept full process exit on close, including `--minimized` startup; `Hide to tray` remains the only background path.

### Build
- Rebuilt Windows x64 portable and installer executables with the corrected close hook.
- Ignored local npm dependency artifacts so release builds no longer dirty the working tree.

### Downloads
- `Pulse-2.0.2-portable-win-x64.exe` — standalone Tauri executable.
- `Pulse-2.0.2-setup-win-x64.exe` — installer for Windows 10/11 x64.


## [2.0.1] — 2026-11-13
### Fixed
- Painted the raw dashboard shell, navigation, loading card, and live clock before any Tauri command or metric sample.
- Kept metric streaming alive with a dedicated runtime thread and buffered the latest sample so early samples are not lost while the webview listener attaches.
- Preserved close-button exit behavior for normal and `--minimized` startup.
### UI
- Added an always-visible topbar clock/time readout from first paint.
- Kept Manuel's palette, raw anti-polish surface, thin identity borders, distinct CPU/RAM/GPU/disk/network accents, and restrained copper warnings.
### Downloads
- `Pulse-2.0.1-portable-win-x64.exe` — standalone Tauri executable.
- `Pulse-2.0.1-setup-win-x64.exe` — Inno Setup wrapper standalone Tauri executable.
## [2.0.0] — 2026-11-13

### Changed
- Migrated the desktop shell from Iced to Tauri 2 with a raw HTML/CSS/JS frontend.
- Split the workspace into `pulse-core`, `src-tauri`, and an archived `legacy-iced` reference crate.
- Preserved native Windows collectors, configuration, startup integration, tray, close-to-exit behavior, and live metric streaming.
- Applied the anti-polish / raw aesthetic: scanned grain, wobbly hand-drawn outlines, uneven rotations, hard-step jitter, lowercase headings, dashed focus states, and no smooth transitions.
- Kept Manuel's palette exactly: `#0D1821` main background, `#142333` sidebar, `#344966` cards, `#F0F4EF` primary text, `#B4CDED` secondary text and primary buttons, `#0D1821` button text, `#B86F52` copper highlights, and `#52677E` borders.
- Reserved copper for collector warnings and temperatures at or above 80 °C.
- Kept CPU, RAM, GPU, disk, and network visual identities distinct with thinner smoothed SVG graph strokes.

### Downloads
- `Pulse-2.0.0-portable-win-x64.exe` — standalone Tauri executable.
- `Pulse-2.0.0-setup-win-x64.exe` — Inno Setup wrapper for the standalone Tauri executable.
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

### UI
- Applied Pulse dark palette: `#0D1821` background, `#142333` sidebar, `#344966` cards, `#F0F4EF` primary text, `#B4CDED` secondary text/buttons, `#B86F52` copper highlights, and `#52677E` borders.
- Replaced bright blue navigation buttons with subtle selected-state card backgrounds.
- Switched cards to thin identity-colored borders and consistent 10 px rounded corners.
- Reduced sparkline stroke to 1.35 px and smoothed the plotted curve.
- Separated CPU, RAM, GPU, disk, and network visual identities.
- Reserved copper for collector warnings and temperatures at or above 80 °C.
- Added a light variant derived from the same palette while keeping Dark/Light persistence.
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
