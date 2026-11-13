use crate::collectors::pdh::PdhQuery;
use crate::models::{CpuMetrics, MetricSample};
use anyhow::{Context, Result};
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_DWORD,
};

const CPU_TOTAL: &str = "\\Processor(_Total)\\% Processor Time";
const CPU_FREQUENCY: &str = "\\Processor Information(_Total)\\% Processor Performance";

pub struct CpuCollector {
    query: PdhQuery,
    total: Vec<usize>,
    per_core: Vec<usize>,
    frequency: Option<usize>,
    nominal_mhz: Option<f32>,
}

impl CpuCollector {
    pub fn new(logical_cores: usize) -> Result<Self> {
        let mut query = PdhQuery::new()?;
        let mut total = Vec::new();
        if let Ok(index) = query.add_counter(CPU_TOTAL) {
            total.push(index);
        }
        if let Ok(index) = query.add_counter("\\Processor Information(_Total)\\% Processor Time") {
            total.push(index);
        }
        let mut per_core = Vec::with_capacity(logical_cores);
        for index in 0..logical_cores {
            let path = format!("\\Processor({index})\\% Processor Time");
            if let Ok(counter) = query.add_counter(&path) {
                per_core.push(counter);
            }
        }
        let frequency = query.add_counter(CPU_FREQUENCY).ok();
        let nominal_mhz = read_nominal_frequency_mhz().ok();
        Ok(Self {
            query,
            total,
            per_core,
            frequency,
            nominal_mhz,
        })
    }

    pub fn read(&mut self) -> Result<CpuMetrics> {
        self.query.collect().context("PDH CPU collect")?;
        let usage = self
            .total
            .iter()
            .find_map(|i| self.query.double(*i))
            .unwrap_or(0.0)
            .clamp(0.0, 100.0) as f32;
        let per_core_percent = self
            .per_core
            .iter()
            .map(|i| self.query.double(*i).unwrap_or(0.0).clamp(0.0, 100.0) as f32)
            .collect();
        let frequency_mhz = self
            .frequency
            .and_then(|i| self.query.double(i))
            .and_then(|p| self.nominal_mhz.map(|base| base * (p as f32 / 100.0)));
        Ok(CpuMetrics {
            usage_percent: usage,
            per_core_percent,
            frequency_mhz,
            logical_cores: 0,
            name: String::new(),
            temperature_c: None,
        })
    }
}

impl crate::collectors::Collector for CpuCollector {
    fn name(&self) -> &'static str {
        "cpu"
    }

    fn collect(&mut self, sample: &mut MetricSample) -> Result<()> {
        sample.cpu = self.read()?;
        Ok(())
    }
}

fn read_nominal_frequency_mhz() -> Result<f32> {
    const KEY: &str = "HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0";
    const VALUE: &str = "~MHz";
    let subkey: Vec<u16> = KEY.encode_utf16().chain(std::iter::once(0)).collect();
    let value_name: Vec<u16> = VALUE.encode_utf16().chain(std::iter::once(0)).collect();
    let mut key = HKEY::default();
    let status = unsafe {
        RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR::from_raw(subkey.as_ptr()),
            None,
            KEY_READ,
            &mut key,
        )
    };
    if status.is_err() {
        anyhow::bail!("RegOpenKeyExW CPU: {status:?}");
    }
    let mut value_type = REG_DWORD;
    let mut bytes = [0u8; 4];
    let mut size = bytes.len() as u32;
    let status = unsafe {
        RegQueryValueExW(
            key,
            PCWSTR::from_raw(value_name.as_ptr()),
            None,
            Some(&mut value_type),
            Some(bytes.as_mut_ptr()),
            Some(&mut size),
        )
    };
    let _ = unsafe { RegCloseKey(key) };
    if status.is_err() {
        anyhow::bail!("RegQueryValueExW CPU: {status:?}");
    }
    Ok(u32::from_le_bytes(bytes) as f32)
}
