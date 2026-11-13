# Pulse 1.1.0

Pulse is a lightweight native Windows system monitor built with Rust and Iced. It presents a
dashboard-first view of CPU, memory, GPU, disk, network, and process activity, with a system tray
and optional Windows startup integration.

![Pulse status](https://img.shields.io/badge/platform-Windows%2010%2F11-blue)
![Rust](https://img.shields.io/badge/rust-1.80%2B-orange)
![License](https://img.shields.io/badge/license-MIT-green)

## Features

- **CPU** — PDH total utilization, logical core count, estimated frequency from `% Processor Performance` and registry nominal clock.
- **Memory** — used, available, total, and percentage from `GlobalMemoryStatusEx`.
- **GPU** — NVIDIA utilization, VRAM, and temperature through dynamic NVML loading; DXGI adapter identity and dedicated VRAM fallback.
- **Storage** — PDH read/write throughput and per-volume capacity from `GetDiskFreeSpaceExW`.
- **Network** — PDH upload/download byte rates across network interfaces.
- **Processes** — Tool Help enumeration, per-process CPU deltas, working-set memory, sorting and filtering.
- **System** — hostname, OS product name, build number, user, uptime, and total physical memory.
- **Dashboard** — Iced Canvas sparklines, bounded 60–300 second history, eight navigation pages, and Dark/Light mode buttons.
- **Windows integration** — `Shell_NotifyIconW` tray menu, minimize-to-tray, and `HKCU\...\Run` startup setting.
- **Configuration** — TOML refresh interval, history length, process row limit, theme, and startup preferences.

Temperature is reported only when a vendor API exposes it. Standard Windows performance APIs do
not guarantee CPU temperature, and DXGI does not expose utilization or temperature. Pulse shows
`n/a` and a collector note instead of inventing a value.

## Build

Requirements:

- Windows 10 or Windows 11, x86_64.
- Rust 1.80+ with the `x86_64-pc-windows-msvc` target.
- Visual Studio Build Tools with the MSVC linker.
- Optional: NVIDIA driver with `nvml.dll` for full GPU telemetry.
- Optional: Inno Setup 6 (`iscc`) for the installer.

```powershell
git clone https://github.com/Z3PHYRZZ/Pulse.git
cd Pulse
cargo build --release
.\target\release\pulse.exe
```

Run the Windows API smoke tests:

```powershell
cargo test --all-targets --no-fail-fast
```

Build the installer after a release build:

```powershell
iscc installer\pulse.iss
```

## Configuration

Pulse loads `%APPDATA%\Pulse\config.toml`. If the file does not exist, the defaults embedded from
`config.default.toml` are used. Use the Settings page to write the file.

```toml
refresh_ms = 1000
history_seconds = 180
max_process_rows = 300
process_sort = "cpu"
start_minimized = false
minimize_to_tray = true
enable_startup = false
theme = "dark"
```

| Key | Range / values | Meaning |
|---|---|---|
| `refresh_ms` | 250–10000 | Engine collection cadence. |
| `history_seconds` | 10–600 | Sparkline ring-buffer length. |
| `max_process_rows` | integer | Process table cap. |
| `process_sort` | `cpu`, `memory`, `name`, `pid` | Initial process sort. |
| `start_minimized` | bool | Launch without showing the window. |
| `minimize_to_tray` | bool | Tray preference for Hide to tray; the window close button always exits fully. |
| `enable_startup` | bool | Applies the HKCU Run value when saved. |
| `theme` | `dark`, `light` | Persisted by the Dark and Light buttons. |

`--minimized` can be passed on the command line; the startup integration uses it. The window close button shuts down the monitoring engine, tray thread, and process. Use **Hide to tray** for background operation.

## Architecture

```text
Pulse Application
├── Iced GUI
├── Monitoring Engine
│   ├── Scheduler
│   └── Aggregator
├── Collectors
│   ├── CPU / RAM / GPU
│   ├── Disk / Network
│   └── Processes / System
└── Windows Integration
    ├── System Tray
    ├── Startup Manager
    └── Notifications seam
```

Each collector implements the `Collector` trait and writes into one `MetricSample`. The engine
runs on a dedicated OS thread and sends samples over a Tokio channel into an Iced subscription.
The GUI never calls Win32 directly. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md),
[docs/WINDOWS_API.md](docs/WINDOWS_API.md), and [docs/ROADMAP.md](docs/ROADMAP.md).

## Performance targets

These are engineering targets, not guarantees:

| Metric | Target |
|---|---|
| Idle CPU usage | Below 1% average |
| Background RAM | Below 50 MB |
| Dashboard refresh | 1 second |
| Graph history | 60–300 seconds |
| Startup | Below 2 seconds |

The release profile enables LTO, a single codegen unit, panic abort, and symbol stripping.

## License

MIT. Copyright (c) 2026 Z3PHYRZZ. See [LICENSE](LICENSE).

The full MIT text, copyright notice, and permission notice are included in this repository and
in the Inno Setup installer.


