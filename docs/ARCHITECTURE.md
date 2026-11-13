# Pulse — Architecture

Pulse is a native Windows system monitor written in Rust. The monitoring engine lives in
`pulse-core`; the desktop shell is a Tauri 2 app in `src-tauri` with a plain HTML/CSS/JS
frontend in `frontend/`. Each metric family has a collector backed by the Windows API best
suited to it, and collectors are aggregated into one `MetricSample` per refresh tick.

```text
Pulse workspace
├── pulse-core/                     monitoring engine (library)
│   ├── engine/
│   │   ├── scheduler.rs — dedicated OS thread, configurable interval
│   │   └── aggregator.rs — collector fan-out and warning aggregation
│   ├── collectors/
│   │   ├── cpu.rs — PDH `% Processor Time`, registry nominal frequency
│   │   ├── memory.rs — GlobalMemoryStatusEx
│   │   ├── gpu.rs — dynamic NVML + DXGI identity/VRAM fallback
│   │   ├── disk.rs — PDH LogicalDisk throughput + GetDiskFreeSpaceExW
│   │   ├── network.rs — PDH Network Interface byte rates
│   │   ├── processes.rs — Tool Help snapshot + GetProcessTimes + GetProcessMemoryInfo
│   │   └── system.rs — hostname, OS registry values, core count, uptime
│   ├── models/ — MetricSample, SystemInfo, ProcessInfo
│   ├── config/ — config.toml load/save (ConfigState), theme
│   └── windows/ — startup (HKCU Run), close hook, notifications
├── src-tauri/                      Tauri 2 desktop shell (binary `pulse`)
│   ├── lib.rs — commands, setup, metric event stream, tray
│   ├── close.rs — WM_CLOSE subclass so titlebar X exits fully
│   └── tauri.conf.json / capabilities/ — window config and ACL
├── frontend/                       raw HTML/CSS/JS UI (frontendDist)
│   ├── index.html / app.js / styles.css
├── legacy-iced/                    archived Iced 0.13 frontend, reference only
│                                   (kept on disk, excluded from the workspace)
└── tests + pulse-core/tests/       collector smoke tests
```

`MetricSample` travels over a Tokio unbounded channel from the engine thread into a Tauri
command-side stream that emits `pulse://metrics`; the frontend listens via
`__TAURI_INTERNALS__.listen` and also fetches the latest sample on boot with
`latest_sample`. The UI keeps bounded 90-point arrays for sparklines. No Windows API
calls happen in the frontend.

The native Win32 tray is owned solely by Tauri's `TrayIconBuilder`. The engine owns no
windowing; `pulse-core::windows` holds startup integration and the close-hook helper.

### Design constraints

- Low overhead: one engine thread, one metric stream thread, no per-widget polling.
- Configurable cadence: `refresh_ms` defaults to 1000 and is clamped to 250–10000 ms.
- Graceful degradation: failed collectors add warnings rather than killing the app.
- Optional telemetry: CPU and GPU temperature are only reported when a driver path exists.
- Close exits fully: `prevent_close()` + `app.exit(0)` on `CloseRequested`; hide-to-tray
  is an explicit command, not an implicit minimize.
