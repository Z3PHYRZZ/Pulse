# Pulse — Architecture

Pulse is a native Windows system monitor written in Rust. The monitoring engine is deliberately
separate from the GUI: each metric family has a collector backed by the Windows API best suited
to it, and collectors are aggregated into one `MetricSample` per refresh tick.

```text
Pulse Application
├── Iced GUI (src/ui, src/app.rs)
├── Monitoring Engine (src/engine)
│   ├── scheduler.rs — dedicated OS thread, configurable interval
│   └── aggregator.rs — collector fan-out and warning aggregation
├── Collectors (src/collectors)
│   ├── cpu.rs — PDH `% Processor Time`, registry nominal frequency
│   ├── memory.rs — GlobalMemoryStatusEx
│   ├── gpu.rs — dynamic NVML + DXGI identity/VRAM fallback
│   ├── disk.rs — PDH LogicalDisk throughput + GetDiskFreeSpaceExW
│   ├── network.rs — PDH Network Interface byte rates
│   ├── processes.rs — Tool Help snapshot + GetProcessTimes + GetProcessMemoryInfo
│   └── system.rs — hostname, OS registry values, core count, uptime
└── Windows Integration (src/windows)
    ├── startup.rs — HKCU Run value
    ├── tray.rs — Shell_NotifyIconW message-only window
    └── notifications.rs — tracing-backed notification seam
```

`MetricSample` travels over a Tokio unbounded channel into an Iced subscription. The UI keeps
bounded ring buffers (`History`) for sparklines. Nothing in the GUI polls Windows APIs directly.

### Design constraints

- Low overhead: one engine thread, one tray thread, no per-widget polling.
- Configurable cadence: `refresh_ms` defaults to 1000 and is clamped to 250–10000 ms.
- Graceful degradation: failed collectors add warnings rather than killing the app.
- Optional telemetry: CPU and GPU temperature are only reported when a driver path exists.
