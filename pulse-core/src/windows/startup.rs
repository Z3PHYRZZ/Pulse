use anyhow::{anyhow, Result};
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ,
};

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const VALUE_NAME: &str = "Pulse";

pub fn is_enabled() -> bool {
    enabled().unwrap_or(false)
}

pub fn enable() -> Result<()> {
    let exe = std::env::current_exe()?;
    let command = format!("\"{}\" --minimized", exe.display());
    let bytes = wide_bytes(&command);
    let key = open_key(KEY_WRITE)?;
    let status =
        unsafe { RegSetValueExW(key, wide_pcwstr(VALUE_NAME), None, REG_SZ, Some(&bytes)) };
    let _ = unsafe { RegCloseKey(key) };
    if status.is_err() {
        return Err(anyhow!("RegSetValueExW failed: {:?}", status));
    }
    Ok(())
}

pub fn disable() -> Result<()> {
    let key = open_key(KEY_WRITE)?;
    let status = unsafe { RegDeleteValueW(key, wide_pcwstr(VALUE_NAME)) };
    let _ = unsafe { RegCloseKey(key) };
    if status.is_err() {
        return Err(anyhow!("RegDeleteValueW failed: {:?}", status));
    }
    Ok(())
}

fn enabled() -> Result<bool> {
    let key = open_key(KEY_READ)?;
    let mut size = 0u32;
    let mut value_type = REG_SZ;
    let status = unsafe {
        RegQueryValueExW(
            key,
            wide_pcwstr(VALUE_NAME),
            None,
            Some(&mut value_type),
            None,
            Some(&mut size),
        )
    };
    let _ = unsafe { RegCloseKey(key) };
    if status.is_err() {
        return Ok(false);
    }
    Ok(size > 2)
}

fn open_key(access: windows::Win32::System::Registry::REG_SAM_FLAGS) -> Result<HKEY> {
    let mut key = HKEY::default();
    let subkey = wide_pcwstr(RUN_KEY);
    let status = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, subkey, None, access, &mut key) };
    if status.is_err() {
        return Err(anyhow!("RegOpenKeyExW Run key failed: {:?}", status));
    }
    Ok(key)
}

fn wide_pcwstr(value: &str) -> PCWSTR {
    let leaked: &'static [u16] = Box::leak(wide_vec(value).into_boxed_slice());
    PCWSTR::from_raw(leaked.as_ptr())
}

fn wide_vec(value: &str) -> Vec<u16> {
    let mut wide: Vec<u16> = value.encode_utf16().collect();
    wide.push(0);
    wide
}

fn wide_bytes(value: &str) -> Vec<u8> {
    wide_vec(value)
        .into_iter()
        .flat_map(u16::to_le_bytes)
        .collect()
}
