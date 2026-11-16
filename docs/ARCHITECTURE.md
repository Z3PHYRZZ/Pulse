# Pulse — Architecture

Pulse is a native Windows system monitor written in Rust. The monitoring engine lives in
`pulse-core`; the desktop shell is a Tauri 2 app in `src-tauri` with a React 19, TypeScript,
and Astryx frontend in `src/`. Each metric family has a collector backed by the Windows API best
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
├── src/                            React + Astryx UI source
│   ├── components/ — shell, live cards, SVG history
│   ├── hooks/ + lib/ — Tauri bridge, metric state, validation
│   └── pages/ — overview, hardware, processes, settings
├── dist/                           Vite/StyleX output used by frontendDist
├── public/fonts/                   locally hosted Figtree + SIL license
├── legacy-iced/                    archived Iced 0.13 frontend, reference only
│                                   (kept on disk, excluded from the workspace)
└── tests + pulse-core/tests/       collector smoke tests
```

`MetricSample` travels over a Tokio unbounded channel from the engine thread into a Tauri
command-side stream that emits `pulse://metrics`; the frontend listens through `@tauri-apps/api/event`
and also fetches the latest sample on boot with `latest_sample`. A one-second polling fallback
continues if events fail, and `timestamp_ms` rejects duplicate or stale samples. The UI keeps
bounded arrays sized by `history_seconds` for sparklines. No Windows API calls happen in the frontend.

Vite builds local ES modules and CSS into `dist/`; the Astryx StyleX plugin has runtime injection
disabled. The Neutral theme and four locally hosted Figtree weights work under the existing
self-only production CSP without CDN access. Node 22, `npm ci`, and the lockfile are part of the
clean build and CI contract.

The native Win32 tray is owned solely by Tauri's `TrayIconBuilder`. The engine owns no
windowing; `pulse-core::windows` holds startup integration and the close-hook helper.

### Design constraints

- Low overhead: one engine thread, one metric stream thread, no per-widget polling.
- Configurable cadence: `refresh_ms` defaults to 1000 and is clamped to 250–10000 ms.
- Graceful degradation: failed collectors add warnings rather than killing the app.
- Optional telemetry: CPU and GPU temperature are only reported when a driver path exists.
- Close exits fully: `prevent_close()` + `app.exit(0)` on `CloseRequested`; hide-to-tray
  is an explicit command, not an implicit minimize.
