use crate::collectors::system::SystemCollector;
use crate::config::{Config, ConfigState};
use crate::engine::aggregator::Aggregator;
use crate::models::{MetricSample, SystemInfo};
use anyhow::{Context, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub struct EngineHandle {
    pub receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<MetricSample>>>>,
    pub system: Arc<SystemInfo>,
    pub config: Arc<ConfigState>,
    #[allow(dead_code)]
    join: Arc<Mutex<Option<JoinHandle<()>>>>,
    #[allow(dead_code)]
    stop: Arc<AtomicBool>,
}

impl EngineHandle {
    pub fn start(config: ConfigState) -> Result<Self> {
        let system = Arc::new(SystemCollector::new()?.info().clone());
        let config = Arc::new(config);
        let aggregator = Aggregator::new(config.get(), &system)?;
        let (sender, receiver) = mpsc::unbounded_channel();
        let stop = Arc::new(AtomicBool::new(false));
        let engine_stop = stop.clone();
        let engine_config = config.clone();
        let system_for_thread = system.clone();
        let handle = std::thread::Builder::new()
            .name("pulse-engine".to_string())
            .spawn(move || {
                run_loop(
                    aggregator,
                    sender,
                    engine_config,
                    system_for_thread,
                    engine_stop,
                )
            })
            .context("spawning monitoring engine")?;
        Ok(Self {
            receiver: Arc::new(Mutex::new(Some(receiver))),
            system,
            config,
            join: Arc::new(Mutex::new(Some(handle))),
            stop,
        })
    }

    pub fn take_receiver(&self) -> Option<mpsc::UnboundedReceiver<MetricSample>> {
        self.receiver.lock().expect("engine receiver lock").take()
    }

    #[allow(dead_code)]
    pub fn shutdown(&self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.join.lock().expect("engine join lock").take() {
            let _ = handle.join();
        }
    }
}

fn run_loop(
    mut aggregator: Aggregator,
    sender: mpsc::UnboundedSender<MetricSample>,
    config: Arc<ConfigState>,
    system: Arc<SystemInfo>,
    #[allow(dead_code)] stop: Arc<AtomicBool>,
) {
    loop {
        if stop.load(Ordering::Relaxed) {
            tracing::info!("engine stop requested");
            break;
        }
        match aggregator.sample(&system) {
            Ok(sample) => {
                if sender.send(sample).is_err() {
                    tracing::info!("metrics receiver closed; engine stops");
                    break;
                }
            }
            Err(err) => tracing::warn!("sample failed: {err:#}"),
        }
        if wait_for(config.get().refresh_interval(), &stop) {
            break;
        }
    }
}

fn wait_for(interval: Duration, stop: &AtomicBool) -> bool {
    let deadline = Instant::now() + interval;
    loop {
        if stop.load(Ordering::Relaxed) {
            return true;
        }
        let now = Instant::now();
        if now >= deadline {
            return false;
        }
        let remaining = deadline.saturating_duration_since(now);
        std::thread::sleep(remaining.min(Duration::from_millis(50)));
    }
}

#[allow(dead_code)]
pub fn load_config_or_default() -> Result<Config> {
    Config::load().context("loading Pulse config")
}
