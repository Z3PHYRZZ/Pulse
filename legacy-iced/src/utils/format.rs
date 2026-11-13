pub fn format_bytes(value: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut n = value as f64;
    let mut unit = 0usize;
    while n >= 1024.0 && unit + 1 < UNITS.len() {
        n /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", value, UNITS[unit])
    } else {
        format!("{n:.2} {}", UNITS[unit])
    }
}

pub fn format_rate(bytes_per_sec: f64) -> String {
    const UNITS: [&str; 5] = ["B/s", "KiB/s", "MiB/s", "GiB/s", "TiB/s"];
    let mut n = bytes_per_sec.max(0.0);
    let mut unit = 0usize;
    while n >= 1024.0 && unit + 1 < UNITS.len() {
        n /= 1024.0;
        unit += 1;
    }
    format!("{n:.2} {}", UNITS[unit])
}

pub fn percent_text(value: f32) -> String {
    format!("{:.1}%", value.clamp(0.0, 100.0))
}

pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86_400;
    let hours = (seconds % 86_400) / 3_600;
    let minutes = (seconds % 3_600) / 60;
    let secs = seconds % 60;
    if days > 0 {
        format!("{days}d {hours:02}h {minutes:02}m")
    } else {
        format!("{hours:02}h {minutes:02}m {secs:02}s")
    }
}
