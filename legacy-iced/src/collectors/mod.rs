pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod memory;
pub mod network;
pub mod pdh;
pub mod processes;
pub mod system;

use crate::models::MetricSample;
use anyhow::Result;

pub type CollectResult<T> = Result<T>;

pub trait Collector: Send {
    #[allow(dead_code)]
    fn name(&self) -> &'static str;
    fn collect(&mut self, sample: &mut MetricSample) -> CollectResult<()>;
}
