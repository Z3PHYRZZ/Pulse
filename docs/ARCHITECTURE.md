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
│   │   ├── cpu.rs — PDH `% Processor Time`, registry nominal frequency, native architecture
│   │   ├── memory.rs — GlobalMemoryStatusEx
│   │   ├── gpu.rs — all-adapter DXGI enumeration + multi-device NVML + PDH GPU Engine
│   │   ├── disk.rs — PDH LogicalDisk throughput + GetDiskFreeSpaceExW
│   │   ├── network.rs — PDH Network Interface byte rates
│   │   ├── processes.rs — Tool Help snapshot + GetProcessTimes + GetProcessMemoryInfo
│   │   └── system.rs — hostname, OS registry values, core count, uptime, CPU name/architecture
│   ├── models/ — MetricSample (gpu + gpus), SystemInfo, ProcessInfo, GpuVendor
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

### GPU detection pipeline

`GpuCollector::new` enumerates every DXGI adapter once and caches the stable identity: name,
PCI vendor/device ID, LUID-based id, dedicated VRAM, shared system memory, and an integrated flag.
Vendor IDs map to NVIDIA (`0x10DE`), Intel (`0x8086`), AMD (`0x1002`/`0x1022`), Qualcomm
(`0x5143`), and Microsoft (`0x1414`). This covers discrete, integrated, and ARM64 Adreno GPUs in
one adapter list.

Telemetry is layered on each collection tick:

1. NVML devices are loaded once from `%SystemRoot%\System32\nvml.dll` (absolute path) and bound
   to NVIDIA adapters by ordinal. Each tick reads utilization, memory, and temperature per device.
2. A PDH `GPU Engine(*)` wildcard counter supplies utilization for adapters without NVML
   telemetry. Instance names are matched against the adapter name; on a single-adapter system all
   engine values are attributed to that adapter.
3. DXGI memory values remain the baseline; values unavailable from every source stay `None` and
   serialize as `n/a` in the UI.

`MetricSample.gpus` carries the full adapter list. `MetricSample.gpu` keeps the primary adapter
(first discrete, else first) for compatibility with the Overview and history channels.

### CPU architecture

`SystemCollector` uses `GetNativeSystemInfo` so ARM64 hosts report the true native architecture
even when the process runs under emulation. The processor name is read from
`HKLM\HARDWARE\DESCRIPTION\System\CentralProcessor\0`. Both values are copied into each sample's
`CpuMetrics` by `system::ensure`.
- Close exits fully: `prevent_close()` + `app.exit(0)` on `CloseRequested`; hide-to-tray
  is an explicit command, not an implicit minimize.

## Platform architecture

Collector implementations are selected by `#[cfg(target_os)]`. `pulse-core/src/collectors/mod.rs`
exposes a single `platform` facade with identical collector names and signatures, so the
aggregator and engine are platform-neutral. Windows uses `collectors/{cpu,gpu,...}.rs` and
`windows/` (registry startup, Win32 tray, close hook). Linux uses `collectors/linux_impl.rs`
(procfs/sysfs) and `linux/desktop.rs` (XDG autostart); Tauri owns the Linux status item.

Linux bindings use only `libc` for `sysconf` and `statvfs`; all other data comes from plain text
`/proc` and `/sys` reads. The Windows crate is compiled only on Windows targets.
