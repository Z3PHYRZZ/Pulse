# Pulse — Universal GPU + Snapdragon detection (2026-11-18)

## Goal
Replace NVIDIA-only GPU detection with universal multi-vendor detection (Intel, AMD, NVIDIA, integrated, multiple adapters) plus Snapdragon/ARM64 CPU and GPU (Adreno) recognition. Then push, bump release, update README.

## Key paths
- Live engine: `pulse-core/src/` — `collectors/gpu.rs`, `collectors/cpu.rs`, `collectors/system.rs`, `engine/aggregator.rs`, `models/metrics.rs`, `models/system.rs`
- Live app: `src-tauri/src/lib.rs` (serializes MetricSample directly — no bridge change needed)
- Live frontend: `src/types.ts`, `src/pages/Hardware.tsx`, `src/pages/Overview.tsx`, `src/lib/metrics.ts` + tests
- NOTE: `src/*.rs` is the DEAD Iced copy — do NOT edit. `src/*.tsx` is live React — edit it.
- Build/version: root `Cargo.toml`, `src-tauri/tauri.conf.json`, `package.json`, `config.default.toml`, `installer/pulse.iss`, `CHANGELOG.md`, `README.md`
- Windows bindings: `windows` 0.62.2 (`Win32_Graphics_Dxgi`, `Win32_System_Performance`, `Win32_System_SystemInformation` already enabled)
- Remote: `https://github.com/Z3PHYRZZ/Pulse.git` (origin), branch `main`

## Design
- `GpuMetrics` gains `vendor: String` (enum-ish label), `id: String` (LUID hex), `dedicated_system_bytes`, `shared_system_bytes`, `is_integrated: bool`, `architecture: String` (e.g. Adreno). Existing fields kept.
- `MetricSample.gpus: Vec<GpuMetrics>` added; `MetricSample.gpu: GpuMetrics` kept as primary (first discrete adapter, else first adapter) for compatibility.
- `GpuCollector` enumerates ALL DXGI adapters once at `new()`: name, vendor id, VRAM, LUID, flags (integrated = `Flags & DXGI_ADAPTER_FLAG_SOFTWARE` or vendor Intel + shared system memory heuristic — use desc fields).
- NVML: dynamically load all device handles (not only index 0), get each device's UUID/by-pci-bus-id to map to DXGI by LUID (best-effort by index order fallback). Attach name/VRAM/util/temp to matching adapter.
- AMD/Intel/Snapdragon utilization: PDH counter `\GPU Engine(*)\Utilization Percentage` — requires instance-name filtering to the target adapter; because PDH wildcards are hard to attribute per-adapter reliably, use `Win32_PerfFormattedData_GPUPerformanceCounters_GPUEngine` via PDH wildcard and aggregate by matching `engtype_3D`/`engtype_GPU` instance names containing the physical adapter name. Pragmatic approach: sample the wildcard array once, sum values whose instance name contains the adapter name; if no match, leave utilization None (no invented values).
- Snapdragon CPU: `GetNativeSystemInfo` gives `wProcessorArchitecture`. Add `architecture: String` and `is_arm64: bool` to `SystemInfo` and `CpuMetrics`. CPU name from registry stays; on ARM64 label "Snapdragon/ARM64" when processor name lacks a vendor string. GPU: Qualcomm vendor id 0x5143 -> architecture "Adreno" (name already carries Adreno from DXGI).
- Frontend: `GpuMetrics` gains matching fields + `MetricSample.gpus`. Graphics page renders one card per adapter with vendor badge; Overview keeps primary summary. History uses primary `gpu`.

## Done
- (in progress) full codebase read: engine, bridge, frontend, build config, tests, docs, FFI signatures verified against windows-0.62.2 source

## Next
1. Backend: metrics model + system info + gpu collector rewrite + cpu architecture + aggregator + tests
2. Frontend: types + pages + metrics + tests
3. Verify: cargo check/test, npm typecheck/test/build
4. Bump version everywhere, CHANGELOG, README
5. Commit + push + create/update GitHub release with rebuilt portable binary if toolchain permits

## Decisions / notes
- Keep `MetricSample.gpu` for backward compatibility so the Tauri bridge and history code need no structural change.
- Detection is hardware-agnostic via DXGI (covers all Windows GPUs including integrated and ARM64 Adreno); NVML and PDH add telemetry where available.
- No invented values: utilization/temp stay None when no API provides them, consistent with README.
