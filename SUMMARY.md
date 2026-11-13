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

## Next steps
1. Run `cargo test --workspace --no-fail-fast` — capture output (blocked by gate burst).
2. `cargo build --release -p pulse` to confirm the shipped binary.
3. Commit (Co-Authored-By + Claude Code attribution), push to origin/main.
4. Update this file after each major step.
