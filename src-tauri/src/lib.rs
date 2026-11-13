mod close;
// language: Rust, file: lib.rs, target: Windows 11 x86_64, edition 2021
use pulse_core::config::{Config, ConfigState, ThemeMode};
use pulse_core::engine::EngineHandle;
use pulse_core::models::MetricSample;
use pulse_core::windows::startup;
use pulse_core::windows::tray::{Tray, TrayEvent};
use serde::Serialize;
use std::sync::{mpsc, Arc, Mutex};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, State, WindowEvent,
};

pub struct PulseState {
    config: Arc<ConfigState>,
    engine: EngineHandle,
    tray_events: Arc<Mutex<mpsc::Receiver<TrayEvent>>>,
    latest_sample: Arc<Mutex<Option<MetricSample>>>,
}

#[derive(Serialize, Clone)]
pub struct Snapshot {
    pub version: String,
    pub config: Config,
    pub theme: String,
    pub system: pulse_core::models::SystemInfo,
    pub startup_enabled: bool,
    pub minimized: bool,
}

#[tauri::command]
fn snapshot(state: State<'_, PulseState>) -> Snapshot {
    let config = state.config.get();
    Snapshot {
        version: env!("CARGO_PKG_VERSION").to_string(),
        config: config.clone(),
        theme: config.theme_mode().as_str().to_string(),
        system: (*state.engine.system).clone(),
        startup_enabled: startup::is_enabled(),
        minimized: std::env::args().any(|arg| arg == "--minimized"),
    }
}

#[tauri::command]
fn latest_sample(state: State<'_, PulseState>) -> Option<MetricSample> {
    state
        .latest_sample
        .lock()
        .expect("latest sample lock")
        .clone()
}

#[tauri::command]
fn set_theme(theme: String, state: State<'_, PulseState>) -> Result<(), String> {
    let mode = ThemeMode::parse(&theme);
    let mut next = state.config.get();
    next.theme = mode.as_str().to_string();
    state.config.set(next).map_err(|err| format!("{err:#}"))
}

#[tauri::command]
fn save_config(config: Config, state: State<'_, PulseState>) -> Result<(), String> {
    state.config.set(config).map_err(|err| format!("{err:#}"))
}

#[tauri::command]
fn set_startup(enabled: bool) -> Result<(), String> {
    if enabled {
        startup::enable()
    } else {
        startup::disable()
    }
    .map_err(|err| format!("{err:#}"))
}

#[tauri::command]
fn open_config_folder() -> Result<(), String> {
    let path = Config::path().map_err(|err| format!("{err:#}"))?;
    let parent = path.parent().unwrap_or(&path);
    std::fs::create_dir_all(parent).map_err(|err| format!("{err:#}"))?;
    std::process::Command::new("explorer.exe")
        .arg(parent)
        .spawn()
        .map_err(|err| format!("{err:#}"))?;
    Ok(())
}

#[tauri::command]
fn hide_window(window: tauri::Window) -> Result<(), String> {
    window.hide().map_err(|err| format!("{err:#}"))
}

#[tauri::command]
fn exit_app(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

fn start_metric_stream(
    app: AppHandle,
    receiver: tokio::sync::mpsc::UnboundedReceiver<MetricSample>,
    latest_sample: Arc<Mutex<Option<MetricSample>>>,
) {
    std::thread::Builder::new()
        .name("pulse-tauri-metric-stream".to_string())
        .spawn(move || {
            let runtime = match tokio::runtime::Runtime::new() {
                Ok(runtime) => runtime,
                Err(err) => {
                    tracing::error!("metric stream runtime failed: {err}");
                    return;
                }
            };
            runtime.block_on(async move {
                let mut receiver = receiver;
                while let Some(sample) = receiver.recv().await {
                    *latest_sample.lock().expect("latest sample lock") = Some(sample.clone());
                    if app.emit("pulse://metrics", sample).is_err() {
                        break;
                    }
                }
            });
        })
        .expect("spawning Pulse metric stream");
}

fn start_tray_stream(app: AppHandle, receiver: Arc<Mutex<mpsc::Receiver<TrayEvent>>>) {
    std::thread::Builder::new()
        .name("pulse-tauri-tray-stream".to_string())
        .spawn(move || loop {
            let event = {
                let guard = receiver.lock().expect("tray receiver lock");
                guard
                    .recv_timeout(std::time::Duration::from_millis(250))
                    .ok()
            };
            match event {
                Some(TrayEvent::Show) => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                Some(TrayEvent::Hide) => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.minimize();
                    }
                }
                Some(TrayEvent::Exit) => {
                    app.exit(0);
                    break;
                }
                None => {}
            }
        })
        .expect("spawning tray stream");
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show Pulse", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide to tray", true, None::<&str>)?;
    let exit = MenuItem::with_id(app, "exit", "Exit", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(app, &[&show, &hide, &separator, &exit])?;
    let icon = app
        .default_window_icon()
        .cloned()
        .unwrap_or_else(|| tauri::image::Image::new(&[], 0, 0));
    let _tray = TrayIconBuilder::with_id("pulse-tray")
        .icon(icon)
        .tooltip("Pulse — system monitor")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.minimize();
                }
            }
            "exit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick { .. } = event {
                if let Some(window) = tray.app_handle().get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("pulse=info")),
        )
        .with_target(false)
        .init();

    tauri::Builder::default()
        .setup(|app| {
            let config = ConfigState::load().map_err(|err| err.to_string())?;
            let engine = EngineHandle::start(config).map_err(|err| err.to_string())?;
            let (tray_sender, tray_receiver) = mpsc::channel();
            let tray = Tray::spawn(tray_sender).map_err(|err| err.to_string())?;
            let latest_sample = Arc::new(Mutex::new(None::<MetricSample>));
            app.manage(PulseState {
                config: engine.config.clone(),
                engine,
                tray_events: Arc::new(Mutex::new(tray_receiver)),
                latest_sample: latest_sample.clone(),
            });
            app.manage(tray);
            let state = app.state::<PulseState>();
            let metrics_receiver = state
                .engine
                .take_receiver()
                .ok_or_else(|| "metrics receiver already taken".to_string())?;
            start_metric_stream(app.handle().clone(), metrics_receiver, latest_sample);
            start_tray_stream(app.handle().clone(), state.tray_events.clone());
            build_tray(app.handle())?;
            close::install_close_hook();
            let window = app
                .get_webview_window("main")
                .ok_or_else(|| "main window missing".to_string())?;
            if std::env::args().any(|arg| arg == "--minimized") {
                let _ = window.minimize();
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { .. } = event {
                window.app_handle().exit(0);
            }
        })
        .invoke_handler(tauri::generate_handler![
            snapshot,
            latest_sample,
            set_theme,
            save_config,
            set_startup,
            open_config_folder,
            hide_window,
            exit_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running Pulse");
}
