# Pulse — Roadmap

| Phase | Scope | Status |
|---|---|---|
| 1 | Cargo project, Iced dashboard, navigation, architecture | Complete |
| 2 | CPU, RAM, process enumeration, one-second metric stream | Complete |
| 3 | GPU/NVML, DXGI fallback, disk and network PDH collectors | Complete |
| 4 | Canvas sparklines, bounded history, process sort/filter | Complete |
| 5 | Native system tray, minimize-to-tray, HKCU startup setting | Complete |
| 6 | Release profile, Inno Setup script, MIT compliance, README, CI | Complete |

## Deferred

- Dynamic island and floating widgets.
- Hardware threshold alerts.
- Benchmark mode and persistent resource history.
- Custom overlay themes.
- Toast notifications through `windows::UI::Notifications` instead of the tracing seam.
