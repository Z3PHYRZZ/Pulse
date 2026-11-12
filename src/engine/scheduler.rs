use crate::collectors::system::SystemCollector;
use crate::config::{Config, ConfigState};
use crate::engine::aggregator::Aggregator;
use crate::models::{MetricSample, SystemInfo};
use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

#[derive(Debug)]
struct EngineThread {
    handle: std::thread::JoinHandle<()>,
}

#[derive(Debug, Clone)]
pub struct EngineHandle {
    pub receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<MetricSample>>>>,
    pub system: Arc<SystemInfo>,
    pub config: Arc<ConfigState>,
    join: Arc<Mutex<Option<EngineThread>>>,
}

impl EngineHandle {
    pub fn start(config: ConfigState) -> Result<Self> {
        let system = Arc::new(SystemCollector::new()?.info().clone());
        let config = Arc::new(config);
        let aggregator = Aggregator::new(config.get(), &system)?;
        let (sender, receiver) = mpsc::unbounded_channel();
        let engine_config = config.clone();
        let system_for_thread = system.clone();
        let handle = std::thread::Builder::new()
            .name("pulse-engine".to_string())
            .spawn(move || run_loop(aggregator, sender, engine_config, system_for_thread))
            .context("spawning monitoring engine")?;
        Ok(Self {
            receiver: Arc::new(Mutex::new(Some(receiver))),
            system,
            config,
            join: Arc::new(Mutex::new(Some(EngineThread { handle }))),
        })
    }

    pub fn take_receiver(&self) -> Option<mpsc::UnboundedReceiver<MetricSample>> {
        self.receiver.lock().expect("engine receiver lock").take()
    }

    pub fn shutdown(&self) {
        if let Some(thread) = self.join.lock().expect("engine join lock").take() {
            let _ = thread.handle.join();
        }
    }
}

fn run_loop(
    mut aggregator: Aggregator,
    sender: mpsc::UnboundedSender<MetricSample>,
    config: Arc<ConfigState>,
    system: Arc<SystemInfo>,
) {
    loop {
        match aggregator.sample(&system) {
            Ok(sample) => {
                if sender.send(sample).is_err() {
                    tracing::info!("metrics receiver closed; engine stops");
                    break;
                }
            }
            Err(err) => tracing::warn!("sample failed: {err:#}"),
        }
        std::thread::sleep(config.get().refresh_interval());
    }
}

#[allow(dead_code)]
pub fn load_config_or_default() -> Result<Config> {
    Config::load().context("loading Pulse config")
}
