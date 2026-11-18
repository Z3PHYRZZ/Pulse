# Pulse 2.2.0

Pulse is a lightweight native system monitor for Windows and Linux, built with Tauri 2, Rust, React 19, and Meta's Astryx design system. It presents a dashboard-first view of CPU, memory, GPU, disk, network, and process activity, with a system tray and optional startup integration.

![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20Linux-blue)
![Architectures](https://img.shields.io/badge/architecture-x64%20%7C%20ARM64-blueviolet)
![Rust](https://img.shields.io/badge/rust-1.80%2B-orange)
![License](https://img.shields.io/badge/license-MIT-green)

## Features

- **CPU** — PDH total utilization, logical core count, estimated frequency from `% Processor Performance` and registry nominal clock. Native architecture detection (x64 / x86 / ARM64), Snapdragon recognition, and registry processor name.
- **Memory** — used, available, total, and percentage from `GlobalMemoryStatusEx`.
- **GPU** — universal multi-vendor detection via DXGI: NVIDIA, Intel, AMD, integrated graphics, multiple simultaneous adapters, and Qualcomm Adreno on Snapdragon. Per-adapter vendor, architecture, type, VRAM, shared memory, temperature, and telemetry source.
- **Storage** — PDH read/write throughput and per-volume capacity from `GetDiskFreeSpaceExW`.
- **Network** — PDH upload/download byte rates across network interfaces.
- **Processes** — Tool Help enumeration, per-process CPU deltas, working-set memory, sorting and filtering.
- **System** — hostname, OS product name, build number, user, uptime, total physical memory, and native processor architecture.
- **Dashboard** — accessible Astryx components, local SVG sparklines, configurable bounded history, eight navigation pages, and Neutral Dark/Light themes.
- **Interface** — locally hosted Figtree typography, semantic status colors, tabular metrics, keyboard focus, reduced-motion support, and responsive navigation.
- **Windows integration** — Tauri tray menu with Show, Hide, and Exit, explicit hide-to-tray, and `HKCU\...\Run` startup setting.
- **Configuration** — TOML refresh interval, history length, process row limit, theme, and startup preferences.

Temperature is reported only when a vendor API exposes it. Standard Windows performance APIs do not guarantee CPU temperature, and DXGI does not expose utilization or temperature. Pulse shows `n/a` and a collector note instead of inventing a value.

## GPU and CPU detection

Pulse enumerates **all** DXGI adapters at startup, so integrated and discrete GPUs from different vendors appear together. The Graphics page renders a card per adapter, and the Overview keeps a primary-adapter summary with an adapter count. Each adapter is identified by vendor PCI ID, device ID, LUID, VRAM, and system-memory budget. Integrated GPUs are flagged from the DXGI software flag, vendor, and adapter name.

| Vendor | PCI ID |
|---|---|
| NVIDIA | `0x10DE` |
| Intel | `0x8086` |
| AMD / ATI | `0x1002`, `0x1022` |
| Qualcomm (Adreno) | `0x5143` |
| Microsoft (basic render / WARP) | `0x1414` |

Telemetry is layered per adapter. DXGI always supplies identity and memory; the other sources fill in utilization and temperature where available.

| Source | Provides | Applies to |
|---|---|---|
| DXGI | Name, vendor, VRAM, shared system memory, integrated flag | All adapters, including Adreno |
| NVML (all devices) | Utilization, dedicated VRAM, temperature | NVIDIA adapters |
| PDH `GPU Engine` | Utilization | Intel, AMD, and integrated adapters |

NVML is loaded by absolute path from `%SystemRoot%\System32\nvml.dll`, so a DLL planted in the application directory cannot be picked up through the search order. NVIDIA devices are bound to their DXGI adapters by ordinal; no shared identifier exists between NVML and DXGI. PDH GPU Engine instance attribution to a specific adapter is best-effort on multi-adapter systems.

CPU architecture comes from `GetNativeSystemInfo` (the true native view, not the WOW64 emulation view), so Snapdragon/ARM64 hosts report `ARM64` and the registry processor name, such as Snapdragon X Elite, is shown directly. Values unavailable from every source stay `n/a`.

### Linux telemetry sources

On Linux the collectors read procfs and sysfs directly. No helper daemon or privileged service is required.

| Metric | Source |
|---|---|
| CPU usage / per-core | `/proc/stat` deltas |
| CPU frequency | `/proc/cpuinfo` |
| Memory | `/proc/meminfo` (`MemAvailable` when present) |
| Disk I/O and volumes | `/proc/diskstats` and `statvfs` |
| Network | `/proc/net/dev` |
| Processes | `/proc/<pid>/stat`, `statm`, and `comm` |
| CPU temperature | `/sys/class/thermal/thermal_zone*/temp` |
| GPU identity and VRAM | `/sys/class/drm/card*/device` (PCI vendor/device IDs, `mem_info_vram_*`) |
| GPU utilization | `gpu_busy_percent` in DRM sysfs, or NVML for NVIDIA |
| GPU temperature | `/sys/class/hwmon/hwmon*/temp*_input` matched by sensor name, or NVML |
| Architecture | Native target architecture with Snapdragon/aarch64 recognition |
| Startup | XDG `~/.config/autostart` desktop entry |

NVML is loaded from absolute system paths only (`/usr/lib*/libnvidia-ml.so.1`), avoiding `LD_LIBRARY_PATH` hijacking. Values a kernel or driver does not expose stay `n/a`.

## Download

Prebuilt Windows x64 and Linux x64 binaries are published on the [v2.2.0 release page](https://github.com/Z3PHYRZZ/Pulse/releases/tag/v2.2.0).

| File | Use |
|---|---|
| `Pulse-2.2.0-setup-win-x64.exe` | x64 installer with Start Menu shortcut, optional desktop icon, README, license, default config, and app icon. |
| `Pulse-2.2.0-x64.msi` | x64 MSI installer for managed deployment. |
| `Pulse-2.2.0-portable-win-x64.exe` | Standalone x64 `pulse.exe`; no installer required. |
| `Pulse-2.2.0-x64.AppImage` | Portable Linux x64 AppImage; no installation required, works on most glibc distributions. |

The published binaries target Windows 10/11 x64 and Linux x64. The engine also builds on Windows ARM64 (Snapdragon) and Linux aarch64, detecting the native architecture through `GetNativeSystemInfo` or `uname`-equivalent data. NVIDIA telemetry requires an NVIDIA driver exposing `nvml.dll` (Windows) or `libnvidia-ml.so.1` (Linux). CPU and GPU temperatures remain hardware-dependent and show `n/a` when unavailable.

The Linux AppImage bundles the application and all userspace libraries; it needs only a glibc-based x64 distribution with WebKit2GTK 4.1 available from the host. Run it with `chmod +x Pulse-2.2.0-x64.AppImage && ./Pulse-2.2.0-x64.AppImage`.

## Design

Pulse uses Astryx's Neutral light/dark theme and actual Astryx React components. Five categorical accents distinguish CPU, memory, GPU, disk, and network graphs. Figtree is served locally under the SIL Open Font License; production needs no CDN or external font connection.

## Build

Requirements:

- Windows 10/11 (x64 or ARM64/Snapdragon) or Linux (x64 or aarch64).
- Node.js 22+ and npm.
- Rust 1.80+ with the target for your platform (`x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`, `x86_64-unknown-linux-gnu`, or `aarch64-unknown-linux-gnu`).
- Visual Studio Build Tools with the MSVC linker.
- Optional: NVIDIA driver with `nvml.dll` in `%SystemRoot%\System32` for full NVIDIA GPU telemetry.
- On Linux: `libwebkit2gtk-4.1-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `libxdo-dev`, `libssl-dev`, and `pkg-config` (the Linux CI workflow installs the full set).
- Optional: Inno Setup 6 (`iscc`) for the Windows installer.

```powershell
git clone https://github.com/Z3PHYRZZ/Pulse.git
cd Pulse
npm ci
npm run typecheck
npm test
npm run tauri build
.\target\release\pulse.exe
```

Run the Windows API smoke tests:

```powershell
cargo test --workspace --all-targets --no-fail-fast
```

Build the installer after a release build:

```powershell
npm run tauri build
```

## Configuration

Pulse loads `%APPDATA%\Pulse\config.toml`. If the file does not exist, the defaults embedded from `config.default.toml` are used. Use the Settings page to write the file.

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
├── React + Astryx GUI (src/, generated dist/)
├── Tauri 2 shell (src-tauri/)
├── Monitoring Engine
│   ├── Scheduler
│   └── Aggregator
├── Collectors
│   ├── CPU / RAM / GPU (DXGI + NVML + PDH)
│   ├── Disk / Network
│   └── Processes / System
└── Windows Integration
    ├── System Tray
    ├── Startup Manager
    └── Notifications seam
```

Each collector implements the `Collector` trait and writes into one `MetricSample`. The engine runs on a dedicated OS thread and sends samples over a Tokio channel into the Tauri `pulse://metrics` event stream. The React frontend uses the official Tauri API, with a one-second `latest_sample` fallback and timestamp deduplication. The GUI never calls Win32 directly. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/WINDOWS_API.md](docs/WINDOWS_API.md), and [docs/ROADMAP.md](docs/ROADMAP.md).

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

The full MIT text, copyright notice, and permission notice are included in this repository and in the Inno Setup installer.
