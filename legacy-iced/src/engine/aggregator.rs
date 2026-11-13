use crate::collectors::cpu::CpuCollector;
use crate::collectors::disk::DiskCollector;
use crate::collectors::gpu::GpuCollector;
use crate::collectors::memory::MemoryCollector;
use crate::collectors::network::NetworkCollector;
use crate::collectors::processes::ProcessCollector;
use crate::collectors::system::{apply_uptime, ensure};
use crate::collectors::Collector;
use crate::config::Config;
use crate::models::{MetricSample, SystemInfo};
use anyhow::{Context, Result};

pub struct Aggregator {
    cpu: CpuCollector,
    memory: MemoryCollector,
    gpu: GpuCollector,
    disk: DiskCollector,
    network: NetworkCollector,
    processes: ProcessCollector,
    warnings: Vec<String>,
}

impl Aggregator {
    pub fn new(_config: Config, system: &SystemInfo) -> Result<Self> {
        let cpu = CpuCollector::new(system.logical_cores).context("CPU collector")?;
        let mut gpu = GpuCollector::new(system.total_memory_bytes).context("GPU collector")?;
        let probe = gpu.read();
        let mut warnings = Vec::new();
        if probe.usage_percent.is_none() {
            warnings.push("GPU utilization and temperature need NVML or vendor telemetry; DXGI reports identity and VRAM only.".to_string());
        }
        let disk = DiskCollector::new().context("disk collector")?;
        let network = NetworkCollector::new().context("network collector")?;
        Ok(Self {
            cpu,
            memory: MemoryCollector::new(),
            gpu,
            disk,
            network,
            processes: ProcessCollector::new(system.logical_cores),
            warnings,
        })
    }

    pub fn sample(&mut self, system: &SystemInfo) -> Result<MetricSample> {
        let mut sample = MetricSample {
            timestamp_ms: crate::utils::platform::now_ms(),
            ..MetricSample::default()
        };
        let collectors: [(&str, &mut dyn Collector); 6] = [
            ("cpu", &mut self.cpu),
            ("memory", &mut self.memory),
            ("gpu", &mut self.gpu),
            ("disk", &mut self.disk),
            ("network", &mut self.network),
            ("processes", &mut self.processes),
        ];
        for (name, collector) in collectors {
            if let Err(err) = collector.collect(&mut sample) {
                tracing::warn!("collector {name} failed: {err:#}");
                sample.warnings.push(format!("{name}: {err}"));
            }
        }
        ensure(system, &mut sample);
        apply_uptime(&mut sample);
        sample.warnings.extend(self.warnings.iter().cloned());
        Ok(sample)
    }
}
