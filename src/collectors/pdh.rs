use anyhow::{anyhow, Result};
use std::mem::zeroed;
use windows::core::PCWSTR;
use windows::Win32::System::Performance::{
    PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterValue,
    PdhOpenQueryW, PDH_FMT_COUNTERVALUE, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY,
};

pub struct PdhQuery {
    handle: PDH_HQUERY,
    counters: Vec<PdhCounter>,
}

pub struct PdhCounter {
    handle: PDH_HCOUNTER,
}

impl PdhQuery {
    pub fn new() -> Result<Self> {
        let mut handle = PDH_HQUERY::default();
        let status = unsafe { PdhOpenQueryW(PCWSTR::null(), 0, &mut handle) };
        if status != 0 {
            return Err(anyhow!("PdhOpenQueryW failed: 0x{status:08X}"));
        }
        Ok(Self {
            handle,
            counters: Vec::new(),
        })
    }

    pub fn add_counter(&mut self, path: &str) -> Result<usize> {
        let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let mut handle = PDH_HCOUNTER::default();
        let status = unsafe {
            PdhAddEnglishCounterW(self.handle, PCWSTR::from_raw(wide.as_ptr()), 0, &mut handle)
        };
        if status != 0 {
            return Err(anyhow!(
                "PdhAddEnglishCounterW failed for '{path}': 0x{status:08X}"
            ));
        }
        let index = self.counters.len();
        self.counters.push(PdhCounter { handle });
        Ok(index)
    }

    pub fn collect(&self) -> Result<()> {
        let status = unsafe { PdhCollectQueryData(self.handle) };
        if status != 0 {
            return Err(anyhow!("PdhCollectQueryData failed: 0x{status:08X}"));
        }
        Ok(())
    }

    pub fn double(&self, index: usize) -> Option<f64> {
        let counter = self.counters.get(index)?;
        let mut value: PDH_FMT_COUNTERVALUE = unsafe { zeroed() };
        let status = unsafe {
            PdhGetFormattedCounterValue(counter.handle, PDH_FMT_DOUBLE, None, &mut value)
        };
        if status != 0 || value.CStatus != 0 {
            return None;
        }
        let number = unsafe { value.Anonymous.doubleValue };
        number.is_finite().then_some(number)
    }
}

impl Drop for PdhQuery {
    fn drop(&mut self) {
        unsafe { PdhCloseQuery(self.handle) };
    }
}

unsafe impl Send for PdhQuery {}
