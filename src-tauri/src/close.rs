// language: Rust, file: close.rs, target: Windows 11 x86_64, edition 2021
// WM_CLOSE hook: subclasses each Pulse top-level window so the titlebar X
// exits fully instead of leaving the WebView process alive. Tauri's own
// CloseRequested handler (see lib.rs) performs the graceful exit; this hook
// exists only to guarantee Win32 WM_CLOSE reaches that handler on builds
// where wry does not forward it. One prev-wndproc per HWND, never shared.
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, EnumThreadWindows, GetWindowTextW, GetWindowThreadProcessId,
    SetWindowLongPtrW, GWLP_WNDPROC, WM_CLOSE,
};

type WndProc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

static PREV_PROCS: LazyLock<Mutex<HashMap<isize, isize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

pub fn install_close_hook() {
    let main_thread_id = unsafe { GetCurrentThreadId() };
    std::thread::Builder::new()
        .name("pulse-tauri-close-hook".to_string())
        .spawn(move || install_close_hook_worker(main_thread_id))
        .expect("spawning Pulse close hook thread");
}

fn install_close_hook_worker(window_thread_id: u32) {
    let current_pid = unsafe { GetCurrentProcessId() };
    // Wait for wry/tao to create the top-level windows (up to 5s).
    for _ in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        let mut hooked = Vec::new();
        unsafe { enum_pulse_windows(current_pid, window_thread_id, &mut hooked) };
        for hwnd in hooked {
            unsafe { hook_window(hwnd) };
        }
        if PREV_PROCS
            .lock()
            .map(|procs| !procs.is_empty())
            .unwrap_or(false)
        {
            return;
        }
    }
    tracing::warn!("Pulse main window not found for WM_CLOSE hook");
}

struct EnumState {
    pid: u32,
    procs: Vec<HWND>,
}

unsafe fn enum_pulse_windows(pid: u32, thread_id: u32, out: &mut Vec<HWND>) {
    let mut state = EnumState { pid, procs: Vec::new() };
    let _ = EnumThreadWindows(
        thread_id,
        Some(enum_pulse_window_callback),
        LPARAM(&mut state as *mut EnumState as isize),
    );
    out.extend(state.procs);
}

unsafe extern "system" fn enum_pulse_window_callback(
    hwnd: HWND,
    lparam: LPARAM,
) -> windows::core::BOOL {
    let state = &mut *(lparam.0 as *mut EnumState);
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid != state.pid {
        return windows::core::BOOL(1);
    }
    let mut title = [0u16; 256];
    let len = GetWindowTextW(hwnd, &mut title) as usize;
    if len == 0 {
        return windows::core::BOOL(1);
    }
    let title = String::from_utf16_lossy(&title[..len]);
    if !title.starts_with("Pulse") {
        return windows::core::BOOL(1);
    }
    state.procs.push(hwnd);
    windows::core::BOOL(1)
}

unsafe fn hook_window(hwnd: HWND) {
    let key = hwnd.0 as isize;
    if let Ok(procs) = PREV_PROCS.lock() {
        if procs.contains_key(&key) {
            return;
        }
    }
    let prev = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, close_wndproc as WndProc as isize);
    if prev == 0 {
        return;
    }
    if let Ok(mut procs) = PREV_PROCS.lock() {
        procs.insert(key, prev);
    }
    tracing::info!("Pulse WM_CLOSE hook installed on {hwnd:?}");
}

unsafe extern "system" fn close_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let prev = PREV_PROCS
        .lock()
        .ok()
        .and_then(|procs| procs.get(&(hwnd.0 as isize)).copied())
        .unwrap_or(0);
    if msg == WM_CLOSE {
        // Let tao/wry receive WM_CLOSE; Tauri's CloseRequested handler exits.
        // Do NOT call ExitProcess here — that skips tray cleanup and webview shutdown.
    }
    if prev == 0 {
        return LRESULT(0);
    }
    let prev_proc = std::mem::transmute::<isize, WndProc>(prev);
    CallWindowProcW(Some(prev_proc), hwnd, msg, wparam, lparam)
}
