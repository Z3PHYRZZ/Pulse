// language: Rust, file: main.rs, target: Windows 11 x86_64, edition 2021
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod collectors;
mod config;
mod engine;
mod messages;
mod models;
mod ui;
mod utils;
#[cfg(windows)]
mod windows;

use anyhow::{Context, Result};
use app::PulseApp;
use config::{Config, ConfigState};
use engine::EngineHandle;
use iced::{Size, Subscription};
use messages::{Message, TrayEvent};
use models::MetricSample;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("pulse=info")),
        )
        .with_target(false)
        .init();

    let runtime = Runtime::new().context("creating Tokio runtime")?;
    let _guard = runtime.enter();

    let config_state = ConfigState::load().context("configuration")?;
    let config_path = Config::path()?;
    let initial_config = config_state.get();
    let engine = EngineHandle::start(config_state).context("monitoring engine")?;

    let (tray_sender, tray_receiver) = std::sync::mpsc::channel();
    let tray = crate::windows::tray::Tray::spawn(tray_sender).context("system tray")?;
    let app = PulseApp::new(engine, config_path, tray_receiver).context("application state")?;

    let minimized = std::env::args().any(|arg| arg == "--minimized");
    let window_settings = iced::window::Settings {
        size: Size::new(1180.0, 760.0),
        visible: !minimized && !initial_config.start_minimized,
        exit_on_close_request: !initial_config.minimize_to_tray,
        ..iced::window::Settings::default()
    };

    tracing::info!("Pulse starting; minimized={minimized}");
    iced::application(PulseApp::title, PulseApp::update, PulseApp::view)
        .subscription(subscriptions)
        .theme(|_| iced::Theme::Dark)
        .window(window_settings)
        .centered()
        .run_with(move || {
            let mut state = app;
            if minimized {
                state.visible = false;
            }
            (
                state,
                iced::window::get_oldest().map(Message::WindowDiscovered),
            )
        })
        .context("Iced application")?;

    tray.shutdown();
    Ok(())
}

fn subscriptions(state: &PulseApp) -> Subscription<Message> {
    Subscription::batch([
        metrics_subscription(state),
        Subscription::run_with_id("pulse-tray", tray_stream(state.tray_receiver.clone())),
        iced::window::close_requests().map(Message::CloseRequested),
    ])
}

fn metrics_subscription(state: &PulseApp) -> Subscription<Message> {
    let receiver = state.engine.take_receiver();
    let id = state.engine.system.hostname.clone();
    Subscription::run_with_id(
        id,
        futures::stream::unfold(
            receiver,
            move |mut receiver: Option<mpsc::UnboundedReceiver<MetricSample>>| async move {
                match receiver.as_mut() {
                    Some(rx) => match rx.recv().await {
                        Some(sample) => Some((Message::Metrics(sample), receiver)),
                        None => None,
                    },
                    None => None,
                }
            },
        ),
    )
}

fn tray_stream(
    receiver: std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<TrayEvent>>>,
) -> impl futures::Stream<Item = Message> {
    futures::stream::unfold(receiver, |receiver| async move {
        loop {
            let event = {
                let guard = receiver.lock().expect("tray receiver lock");
                messages::drain_tray(&guard)
            };
            if let Some(event) = event {
                return Some((Message::TrayEvent(event), receiver));
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    })
}
