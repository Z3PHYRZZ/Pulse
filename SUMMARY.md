# Pulse — Universal GPU + Snapdragon detection (2026-11-18)

## Goal
Universal GPU detection (Intel/AMD/NVIDIA/integrated/multi-adapter/Adreno) + Snapdragon CPU support, then Linux support with an AppImage build.

## Key paths
- Live engine: `pulse-core/src/` — Windows collectors + `collectors/linux_impl.rs`, `linux/desktop.rs`, `collectors/mod.rs` platform facade
- Live app: `src-tauri/src/lib.rs`; `src-tauri/tauri.conf.json` bundles msi/nsis/appimage
- Live frontend: `src/*.tsx` (the `src/*.rs` files are the dead Iced copy)
- Version files: root `Cargo.toml`, `src-tauri/tauri.conf.json`, `package.json`, `config.default.toml`, `installer/pulse.iss`
- CI: `.github/workflows/windows.yml`, `.github/workflows/linux.yml`
- Remote: origin `https://github.com/Z3PHYRZZ/Pulse.git`, branch `main`

## Design
- `GpuMetrics` gains `vendor: String` (enum-ish label), `id: String` (LUID hex), `dedicated_system_bytes`, `shared_system_bytes`, `is_integrated: bool`, `architecture: String` (e.g. Adreno). Existing fields kept.
- `MetricSample.gpus: Vec<GpuMetrics>` added; `MetricSample.gpu: GpuMetrics` kept as primary (first discrete adapter, else first adapter) for compatibility.
- `GpuCollector` enumerates ALL DXGI adapters once at `new()`: name, vendor id, VRAM, LUID, flags (integrated = `Flags & DXGI_ADAPTER_FLAG_SOFTWARE` or vendor Intel + shared system memory heuristic — use desc fields).
- NVML: dynamically load all device handles (not only index 0), get each device's UUID/by-pci-bus-id to map to DXGI by LUID (best-effort by index order fallback). Attach name/VRAM/util/temp to matching adapter.
- AMD/Intel/Snapdragon utilization: PDH counter `\GPU Engine(*)\Utilization Percentage` — requires instance-name filtering to the target adapter; because PDH wildcards are hard to attribute per-adapter reliably, use `Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine` via PDH wildcard and aggregate by matching `engtype_3D`/`engtype_GPU` instance names containing the physical adapter name. Pragmatic approach: sample the wildcard array once, sum values whose instance name contains the adapter name; if no match, leave utilization None (no invented values).
- Snapdragon CPU: `GetNativeSystemInfo` gives `wProcessorArchitecture`. Add `architecture: String` and `is_arm64: bool` to `SystemInfo` and `CpuMetrics`. CPU name from registry stays; on ARM64 label "Snapdragon/ARM64" when processor name lacks a vendor string. GPU: Qualcomm vendor id 0x5143 -> architecture "Adreno" (name already carries Adreno from DXGI).
- Frontend: `GpuMetrics` gains matching fields + `MetricSample.gpus`. Graphics page renders one card per adapter with vendor badge; Overview keeps primary summary. History uses primary `gpu`.

## Done
- v2.1.0: universal GPU detection, Snapdragon support, per-adapter UI, docs, PR #2 merged, release published
- v2.2.0 Linux: per-OS collector facade, procfs/sysfs collectors, XDG autostart, Linux PNG icons, `linux.yml` AppImage CI
- Verified: Windows cargo check/test/release build, frontend typecheck/tests/build, Linux cargo check for pulse-core (x86_64-unknown-linux-gnu), Linux GitHub Actions run 44267077109 green producing the AppImage
- Committed `ea8ce5c`, `e4523be`, `a763f61` and pushed to main; tag v2.2.0 pushed
- GitHub release v2.2.0 published with four assets: AppImage (SHA-256 50447AF379A348A81A054B0B8362EBA5BE146385501C077499C9DA7706770A59), portable exe, NSIS installer, MSI
- Working tree clean, main in sync with origin

## Next
- None outstanding. Future work from ROADMAP: threshold alerts, benchmark mode, persistent history, custom overlay themes, toast notifications.

## Decisions / notes
- Keep `MetricSample.gpu` for backward compatibility so the Tauri bridge and history code need no structural change.
- Detection is hardware-agnostic via DXGI (covers all Windows GPUs including integrated and ARM64 Adreno); NVML and PDH add telemetry where available.
- No invented values: utilization/temp stay None when no API provides them, consistent with README.
