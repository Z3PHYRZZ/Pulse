// language: Rust, file: close.rs, target: Windows 11 x86_64, edition 2021
use std::sync::atomic::{AtomicIsize, AtomicUsize, Ordering};
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
use windows::Win32::UI::WindowsAndMessaging::{
    CallWindowProcW, EnumThreadWindows, GetWindowTextW, GetWindowThreadProcessId,
    SetWindowLongPtrW, GWLP_WNDPROC, WM_CLOSE,
};

type WndProc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

const HOOK_READY: u32 = 0x8000;

static PREV_WNDPROC: [AtomicIsize; 4] = [const { AtomicIsize::new(0) }; 4];
static HOOK_INDEX: AtomicUsize = AtomicUsize::new(0);

pub fn install_close_hook() {
    let main_thread_id = unsafe { GetCurrentThreadId() };
    std::thread::Builder::new()
        .name("pulse-tauri-close-hook".to_string())
        .spawn(move || install_close_hook_worker(main_thread_id))
        .expect("spawning Pulse close hook thread");
}

fn install_close_hook_worker(window_thread_id: u32) {
    let current_pid = unsafe { GetCurrentProcessId() };
    let mut hooked: Vec<HWND> = Vec::new();
    for _ in 0..100 {
        std::thread::sleep(std::time::Duration::from_millis(50));
        while let Some(hwnd) =
            unsafe { find_next_pulse_window(current_pid, window_thread_id, &hooked) }
        {
            if unsafe { hook_window(hwnd, hooked.len()) } {
                hooked.push(hwnd);
                tracing::info!("Pulse WM_CLOSE hook installed on {hwnd:?}");
            }
            if hooked.len() >= 4 {
                return;
            }
        }
    }
}

unsafe fn find_next_pulse_window(pid: u32, window_thread_id: u32, hooked: &[HWND]) -> Option<HWND> {
    let mut target = HookTarget {
        hwnd: None,
        pid,
        hooked,
    };
    let _ = EnumThreadWindows(
        window_thread_id,
        Some(enum_pulse_window),
        LPARAM(&mut target as *mut HookTarget as isize),
    );
    target.hwnd
}

unsafe fn hook_window(hwnd: HWND, index: usize) -> bool {
    let prev = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, close_wndproc as WndProc as isize);
    if prev == 0 {
        return false;
    }
    PREV_WNDPROC[index].store(prev, Ordering::Release);
    true
}

struct HookTarget<'a> {
    hwnd: Option<HWND>,
    pid: u32,
    hooked: &'a [HWND],
}

unsafe extern "system" fn enum_pulse_window(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    let target = &mut *(lparam.0 as *mut HookTarget<'_>);
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid != target.pid || target.hooked.iter().any(|known| known.0 == hwnd.0) {
        return windows::core::BOOL(1);
    }
    let mut title = [0u16; 256];
    let len = GetWindowTextW(hwnd, &mut title) as usize;
    let title = String::from_utf16_lossy(&title[..len]);
    if !title.starts_with("Pulse") {
        return windows::core::BOOL(1);
    }
    target.hwnd = Some(hwnd);
    windows::core::BOOL(0)
}

unsafe extern "system" fn close_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_CLOSE {
        pulse_core::force_exit();
    }
    if msg == HOOK_READY {
        return LRESULT(0);
    }
    let index = HOOK_INDEX.load(Ordering::Acquire);
    let prev = PREV_WNDPROC[index].load(Ordering::Acquire);
    if prev == 0 {
        return LRESULT(0);
    }
    let prev = std::mem::transmute::<isize, WndProc>(prev);
    CallWindowProcW(Some(prev), hwnd, msg, wparam, lparam)
}
