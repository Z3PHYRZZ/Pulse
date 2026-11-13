use crate::collectors::pdh::PdhQuery;
use crate::models::{AdapterTraffic, MetricSample, NetworkMetrics};
use anyhow::{Context, Result};

const NET_SENT: &str = "\\Network Interface(*)\\Bytes Sent/sec";
const NET_RECEIVED: &str = "\\Network Interface(*)\\Bytes Received/sec";

pub struct NetworkCollector {
    query: PdhQuery,
    sent: Option<usize>,
    received: Option<usize>,
}

impl NetworkCollector {
    pub fn new() -> Result<Self> {
        let mut query = PdhQuery::new()?;
        let sent = query.add_counter(NET_SENT).ok();
        let received = query.add_counter(NET_RECEIVED).ok();
        Ok(Self {
            query,
            sent,
            received,
        })
    }

    pub fn read(&mut self) -> Result<NetworkMetrics> {
        self.query.collect().context("PDH network collect")?;
        let upload = self
            .sent
            .and_then(|i| self.query.double(i))
            .unwrap_or(0.0)
            .max(0.0);
        let download = self
            .received
            .and_then(|i| self.query.double(i))
            .unwrap_or(0.0)
            .max(0.0);
        Ok(NetworkMetrics {
            upload_bytes_per_sec: upload,
            download_bytes_per_sec: download,
            adapters: Vec::new(),
        })
    }
}

impl crate::collectors::Collector for NetworkCollector {
    fn name(&self) -> &'static str {
        "network"
    }

    fn collect(&mut self, sample: &mut MetricSample) -> Result<()> {
        sample.network = self.read()?;
        Ok(())
    }
}

#[allow(dead_code)]
pub fn adapter_row(name: String, up: f64, down: f64) -> AdapterTraffic {
    AdapterTraffic {
        name,
        upload_bytes_per_sec: up,
        download_bytes_per_sec: down,
    }
}
