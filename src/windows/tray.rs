use crate::messages::TrayEvent;
use anyhow::{anyhow, Context, Result};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;
use windows::core::PCWSTR;
use windows::Win32::Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::ExitProcess;
use windows::Win32::UI::Shell::{
    Shell_NotifyIconW, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DestroyMenu, DispatchMessageW,
    GetCursorPos, LoadIconW, LoadImageW, PeekMessageW, PostQuitMessage, RegisterClassW,
    SetForegroundWindow, TrackPopupMenu, TranslateMessage, HICON, IDI_APPLICATION, IMAGE_ICON,
    LR_DEFAULTSIZE, LR_LOADFROMFILE, MF_SEPARATOR, MF_STRING, MSG, PM_REMOVE, TPM_RETURNCMD,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_COMMAND, WM_DESTROY, WM_LBUTTONDBLCLK, WM_RBUTTONUP, WM_USER,
    WNDCLASSW,
};

const TRAY_CALLBACK: u32 = WM_USER + 1;
const TRAY_FORCE_EXIT: u32 = WM_USER + 2;
const ID_SHOW: usize = 1001;
const ID_HIDE: usize = 1002;
const ID_EXIT: usize = 1003;
const WM_QUIT: u32 = 0x0012;

pub struct Tray {
    stop: Sender<()>,
    thread: Option<JoinHandle<()>>,
}

impl Tray {
    pub fn spawn(events: Sender<TrayEvent>) -> Result<Self> {
        let (stop_tx, stop_rx) = mpsc::channel();
        let thread = std::thread::Builder::new()
            .name("pulse-tray".to_string())
            .spawn(move || {
                if let Err(err) = tray_loop(events, stop_rx) {
                    tracing::warn!("tray loop failed: {err:#}");
                }
            })
            .context("spawning tray thread")?;
        Ok(Self {
            stop: stop_tx,
            thread: Some(thread),
        })
    }

    pub fn shutdown(mut self) {
        let _ = self.stop.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn tray_loop(events: Sender<TrayEvent>, stop: Receiver<()>) -> Result<()> {
    let class_name = wide_static("PulseTrayWindow");
    let instance = unsafe { GetModuleHandleW(PCWSTR::null()) }.context("GetModuleHandleW")?;
    let window_class = WNDCLASSW {
        lpfnWndProc: Some(tray_wndproc),
        hInstance: HINSTANCE(instance.0),
        lpszClassName: PCWSTR::from_raw(class_name.as_ptr()),
        ..unsafe { std::mem::zeroed() }
    };
    if unsafe { RegisterClassW(&window_class) } == 0 {
        return Err(anyhow!("RegisterClassW failed"));
    }
    let hwnd = unsafe {
        CreateWindowExW(
            WINDOW_EX_STYLE::default(),
            PCWSTR::from_raw(class_name.as_ptr()),
            PCWSTR::from_raw(wide_static("Pulse").as_ptr()),
            WINDOW_STYLE::default(),
            0,
            0,
            0,
            0,
            Some(HWND(-3isize as *mut core::ffi::c_void)),
            None,
            Some(HINSTANCE(instance.0)),
            None,
        )
    }
    .context("CreateWindowExW message-only window")?;
    let icon = load_pulse_icon().unwrap_or_else(|_| {
        unsafe { LoadIconW(None, IDI_APPLICATION) }.expect("LoadIconW fallback")
    });
    let mut icon_data = icon_data(hwnd, icon);
    if !unsafe { Shell_NotifyIconW(NIM_ADD, &icon_data) }.as_bool() {
        return Err(anyhow!("Shell_NotifyIconW(NIM_ADD) failed"));
    }
    let mut msg: MSG = unsafe { std::mem::zeroed() };
    let mut running = true;
    while running && stop.try_recv().is_err() {
        while unsafe { PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE) }.as_bool() {
            if msg.message == WM_QUIT || msg.message == TRAY_FORCE_EXIT {
                running = false;
                break;
            }
            unsafe {
                let _ = TranslateMessage(&msg);
                let _ = DispatchMessageW(&msg);
            }
            let command = match msg.message {
                TRAY_CALLBACK => match (msg.lParam.0 as u32) & 0xFFFF {
                    WM_LBUTTONDBLCLK => Some(TrayEvent::Show),
                    WM_RBUTTONUP => show_menu(hwnd),
                    _ => None,
                },
                WM_COMMAND => match msg.wParam.0 & 0xFFFF {
                    ID_SHOW => Some(TrayEvent::Show),
                    ID_HIDE => Some(TrayEvent::Hide),
                    ID_EXIT => Some(TrayEvent::Exit),
                    _ => None,
                },
                _ => None,
            };
            if let Some(event) = command {
                let is_exit = event == TrayEvent::Exit;
                let _ = events.send(event);
                if is_exit {
                    running = false;
                    break;
                }
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    icon_data.uFlags = Default::default();
    let _ = unsafe { Shell_NotifyIconW(NIM_DELETE, &icon_data) };
    Ok(())
}

fn show_menu(hwnd: HWND) -> Option<TrayEvent> {
    let menu = unsafe { CreatePopupMenu() }.ok()?;
    unsafe {
        AppendMenuW(
            menu,
            MF_STRING,
            ID_SHOW,
            PCWSTR::from_raw(wide_static("Show Pulse").as_ptr()),
        )
        .ok()?;
        AppendMenuW(
            menu,
            MF_STRING,
            ID_HIDE,
            PCWSTR::from_raw(wide_static("Hide Pulse").as_ptr()),
        )
        .ok()?;
        AppendMenuW(menu, MF_SEPARATOR, 0, PCWSTR::null()).ok()?;
        AppendMenuW(
            menu,
            MF_STRING,
            ID_EXIT,
            PCWSTR::from_raw(wide_static("Exit").as_ptr()),
        )
        .ok()?;
        let mut cursor = POINT::default();
        GetCursorPos(&mut cursor).ok()?;
        let _ = SetForegroundWindow(hwnd);
        let command = TrackPopupMenu(menu, TPM_RETURNCMD, cursor.x, cursor.y, None, hwnd, None);
        DestroyMenu(menu).ok()?;
        match command.0 as usize {
            ID_SHOW => Some(TrayEvent::Show),
            ID_HIDE => Some(TrayEvent::Hide),
            ID_EXIT => Some(TrayEvent::Exit),
            _ => None,
        }
    }
}

unsafe extern "system" fn tray_wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_DESTROY {
        PostQuitMessage(0);
        return LRESULT(0);
    }
    DefWindowProcW(hwnd, message, wparam, lparam)
}

fn icon_data(hwnd: HWND, icon: HICON) -> NOTIFYICONDATAW {
    let mut data: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };
    data.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
    data.hWnd = hwnd;
    data.uID = 1;
    data.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
    data.uCallbackMessage = TRAY_CALLBACK;
    data.hIcon = icon;
    for (index, unit) in "Pulse — system monitor"
        .encode_utf16()
        .enumerate()
        .take(127)
    {
        data.szTip[index] = unit;
    }
    data
}

pub fn force_exit() -> ! {
    unsafe { ExitProcess(0) }
}

fn load_pulse_icon() -> Result<HICON> {
    let exe = std::env::current_exe()?;
    let icon_path = exe
        .parent()
        .map(|p| p.join("icons").join("pulse.ico"))
        .unwrap_or_else(|| std::path::PathBuf::from("icons/pulse.ico"));
    if !icon_path.exists() {
        anyhow::bail!("pulse.ico not found beside executable");
    }
    let wide: Vec<u16> = icon_path
        .to_string_lossy()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let handle = unsafe {
        LoadImageW(
            None,
            PCWSTR::from_raw(wide.as_ptr()),
            IMAGE_ICON,
            0,
            0,
            LR_LOADFROMFILE | LR_DEFAULTSIZE,
        )
    }
    .context("LoadImageW pulse.ico")?;
    Ok(HICON(handle.0))
}

fn wide_static(value: &str) -> &'static [u16] {
    let mut wide: Vec<u16> = value.encode_utf16().collect();
    wide.push(0);
    Box::leak(wide.into_boxed_slice())
}
