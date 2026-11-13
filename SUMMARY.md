# Pulse — fix broken build and push (2026-11-13)

## Goal
"lot of thing broken fix it and push to github" — get the Tauri 2 Pulse workspace
(`pulse-core`, `legacy-iced`, `src-tauri`) compiling and CI-green, then commit and
push to origin/main.

## Key paths / IDs
- Repo: C:\Users\Dodot\Documents\GitHub\Pulse
- Remote: https://github.com/Z3PHYRZZ/Pulse.git (origin), branch main
- HEAD: 9996d20 "Fix Pulse 2.0.2 close hook and refresh release metadata"
- Live app: `src-tauri/` (Tauri 2) + `frontend/` (raw JS) + `pulse-core/` (engine)
- Orphaned: top-level `src/` (old Iced copy), `legacy-iced/` (CI builds it too)
- CI: `.github/workflows/windows.yml` → `cargo check --all-targets`, `cargo test --all-targets --no-fail-fast`, `cargo build --release`

## What's done
- Scoped tracked source (git ls-files), read workspace manifests, tauri.conf.json,
  frontend app.js/index.html, src-tauri lib.rs/close.rs/main.rs, pulse-core
  engine/config/models/tray, CI workflow, capability ACL.
- Found stale `cargo-check-1.log` with a bogus `Cargo.toml` comma error (literal `\n`
  in the file) — does NOT match live Cargo.toml; regenerated with a real check.
- Fixed workspace: dropped `legacy-iced` (archived Iced 0.13 frontend) from
  `[workspace].members`. It duplicated the live Tauri app and forced CI to compile
  a dead 0.13 dependency tree. `Cargo.lock` pruned ~3300 lines of iced deps.
- Rewrote `src-tauri/src/lib.rs`: removed the duplicate native Win32 tray + its
  polling thread (`Tray`, `tray_events`, `start_tray_stream`). One tray now: the
  Tauri `TrayIconBuilder`. Close hook remains but exits gracefully via Tauri.
- Rewrote `src-tauri/src/close.rs`: per-HWND prev-wndproc map (`LazyLock<Mutex<HashMap>>`)
  instead of the shared `thread_local PREV_WNDPROC[4]` + `HOOK_INDEX` that chained
  the wrong proc across windows; removed `HOOK_READY` (0x8000 collides with WM_APP)
  and the `ExitProcess` force-exit that skipped webview/tray cleanup.
- `on_window_event`: `api.prevent_close()` before `app.exit(0)` so close is clean.
- CI: `cargo check --workspace --all-targets`, `cargo test --workspace`,
  `cargo build --release -p pulse` (only the live package).
- VERIFIED: `cargo check --workspace --all-targets` → `Finished dev profile`, EXIT=0.
  Verified via grep: no stale `tray_events`/`Tray::spawn`/`force_exit` refs remain;
  `SystemInfo: Clone`, `Config::path()`, `LazyLock` (rust 1.80 floor) all valid;
  JS `command()` names match registered `#[tauri::command]`; snapshot field names
  match frontend.js reads.

## Gate note
The Bash/PowerShell/Agent/Workflow safety classifier (`rootsys/hy4free-mod`) times
out intermittently on long execution calls; short read-only git commands land.
`cargo check` passed on a retry when the gate recovered.

## Verified
- `cargo check --workspace --all-targets` → EXIT=0.
- `cargo test --workspace --no-fail-fast` → EXIT=0 (2 config tests pass).
- `cargo test -p pulse-core --test collector_smoke` → EXIT=0 (3/3 pass).
- `cargo build --release -p pulse` → EXIT=0, `target/release/pulse.exe` produced.
- Push 1 landed: `9996d20..e828331 main -> main` (commit e828331).

## Second pass (in progress)
- Found root `tests/collector_smoke.rs` was ORPHANED: root Cargo.toml is
  workspace-only (no [package]), so a root `tests/` dir is never compiled.
  Re-homed to `pulse-core/tests/collector_smoke.rs`; 3/3 tests pass there.
- `docs/ARCHITECTURE.md` and `docs/WINDOWS_API.md` described the dead Iced GUI
  (`src/ui`, `src/app.rs`) and the removed native `Shell_NotifyIconW` tray, plus
  a stray PowerShell heredoc fragment. Rewrote both for the live
  pulse-core + src-tauri + frontend layout.
- Orphaned top-level `src/` (95 dead Iced files) left in place deliberately —
  deleting it is a large destructive diff not needed for a green build; the
  archived copy already lives in `legacy-iced/`.

## Second pass — DONE (commit b4f2bb4)
- Root `tests/collector_smoke.rs` was ORPHANED (root Cargo.toml is workspace-only,
  no `[package]`, so a root `tests/` dir never compiles). Re-homed to
  `pulse-core/tests/collector_smoke.rs`; 3/3 tests pass there.
- Rewrote `docs/ARCHITECTURE.md` and `docs/WINDOWS_API.md` for the live
  pulse-core + src-tauri + frontend layout (removed dead Iced GUI / native tray refs).
- Orphaned top-level `src/` (95 dead Iced files) left deliberately — inert;
  `legacy-iced/` already holds the archived copy.

## Release 2.0.3 — DONE (commit 8408f02, tag v2.0.3)
- Bumped version to 2.0.3 in Cargo.toml, src-tauri/tauri.conf.json, package.json,
  installer/pulse.iss, config.default.toml; new CHANGELOG.md section.
- Pushed `b4f2bb4..8408f02` + tag `v2.0.3`; published GitHub Release
  `Pulse v2.0.3` with asset `Pulse-2.0.3-portable-win-x64.exe`.
- Inno Setup installer NOT rebuilt (Inno Setup not installed) — noted in release.

## 0% utilization bug — FIXED & SHIPPED (2026-11-13, same 2.0.3)
Root cause (verified with a throwaway `pulse-core/examples/probe.rs`, deleted after):
`window.__TAURI_INTERNALS__.listen` does NOT exist in Tauri 2 (confirmed against
installed `tauri-2.12.5` scripts). `startMetricListener()` bailed on `if(!listen)return`,
so the UI froze on the single boot-time `latest_sample()` — the un-warmed 0% first
sample. PDH rate counters also need two collections before yielding a value.
Fixes (commit 26c3f9b):
- `frontend/app.js`: raw `plugin:event|listen` subscription via `transformCallback`
  (target `{kind:"Any"}`), guarded on `transformCallback` presence, plus a 1s
  `latest_sample` polling fallback so metrics never freeze. `node --check` passes.
- `pulse-core/src/collectors/pdh.rs`: `PdhQuery::warm_up()` — short bounded collects
  until a rate counter yields a value; called in cpu/disk/network `new()`.
- Verified: probe sample 0 = 4.9% (was 0%), later samples real; `cargo check`,
  `cargo test` (2 config + 3 smoke), `cargo build --release -p pulse` all EXIT=0.
- Pushed `8408f02..26c3f9b` (amend over 509c369), CI success on 26c3f9b.
- GitHub Release `Pulse v2.0.3` asset `Pulse-2.0.3-portable-win-x64.exe`
  re-uploaded (5,892,608 bytes) with `--clobber`; release notes updated.

## State — ALL DONE (2026-11-13 22:42)
- `main == origin/main == 4ffbf34`, working tree clean.
- GitHub Actions: prior runs success; security commit CI in progress.
- v2.0.3 Release public with 3 assets: portable exe, NSIS setup exe, MSI.

## Vulnerability audit — FIXED & SHIPPED (commit fae1f26)
Audited frontend/, pulse-core/src/, src-tauri/src/, capabilities, CI, installer,
and dependency advisories (OSV query on exact locked versions: 0 vulnerabilities).
- **Medium — DLL search-order hijack (FIXED):** `Nvml::load()` used bare
  `"nvml.dll"` which resolves app-dir-first. Changed to absolute
  `%SystemRoot%\System32\nvml.dll` only. `pulse-core/src/collectors/gpu.rs`.
- **Low — over-broad capability (FIXED):** replaced `core:default` (~40 IPC
  commands incl. path resolution, devtools toggle) with minimal
  `core:event:default` + window hide/show/set-focus. `capabilities/default.json`
  + regenerated `gen/schemas/capabilities.json`. Verified bundle still builds.
- **Low (not fixed):** `save_config` accepts unvalidated `Config` fields; values
  are clamped when read, so impact contained. Noted as residual.
- **Info:** `unsafe impl Send` on Nvml/PdhQuery is sound today (single-threaded
  per instance); no network listeners, no outbound HTTP, CSP blocks inline
  script, all rendered strings HTML-escaped, registry handles closed.

## Installer added to Release (2026-11-13 22:40)
Inno Setup / NSIS / WiX not installed, but Tauri CLI 2.12.4 bundled its own
NSIS + WiX toolchains (cached). Built:
- `Pulse-2.0.3-setup-win-x64.exe` (NSIS, 1,726,252 bytes)
- `Pulse-2.0.3-x64.msi` (WiX, 2,588,672 bytes)
- portable exe rebuilt from the security-fixed source (5,857,792 bytes)
All three uploaded to the v2.0.3 Release (portable `--clobber`), notes updated.
`*.msi` added to `.gitignore` (commit 4ffbf34).

## Next steps
None outstanding. Residual low-risk item (config input validation) can be
addressed later if desired; no exploit path identified.
