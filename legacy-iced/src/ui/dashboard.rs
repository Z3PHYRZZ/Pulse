use crate::app::PulseApp;
use crate::messages::{Message, Page};
use crate::ui::theme as pulse_theme;
use crate::ui::widgets::{detail_card, metric_card, section_title};
use crate::utils::{format_bytes, format_rate, format_uptime, percent_text};
use iced::widget::{button, column, row, scrollable, text};
use iced::{Element, Length, Theme};

pub fn view(app: &PulseApp) -> Element<'_, Message> {
    let s = &app.latest;
    let cpu = metric_card(
        "CPU",
        percent_text(s.cpu.usage_percent),
        format!("{} logical cores • {}", s.cpu.logical_cores, s.cpu.name),
        &app.history.cpu,
        pulse_theme::CPU,
    );
    let memory = metric_card(
        "Memory",
        percent_text(s.memory.usage_percent),
        format!(
            "{} of {}",
            format_bytes(s.memory.used_bytes),
            format_bytes(s.memory.total_bytes)
        ),
        &app.history.memory,
        pulse_theme::MEMORY,
    );
    let gpu_usage = s
        .gpu
        .usage_percent
        .map(percent_text)
        .unwrap_or_else(|| "n/a".to_string());
    let gpu = metric_card(
        "GPU",
        gpu_usage,
        format!("{} • {}", s.gpu.name, s.gpu.source),
        &app.history.gpu,
        pulse_theme::GPU,
    );
    let disk = metric_card(
        "Disk",
        format_rate(s.disk.read_bytes_per_sec + s.disk.write_bytes_per_sec),
        format!(
            "R {} • W {}",
            format_rate(s.disk.read_bytes_per_sec),
            format_rate(s.disk.write_bytes_per_sec)
        ),
        &app.history.disk,
        pulse_theme::DISK,
    );
    let network = metric_card(
        "Network",
        format_rate(s.network.upload_bytes_per_sec + s.network.download_bytes_per_sec),
        format!(
            "↑ {} • ↓ {}",
            format_rate(s.network.upload_bytes_per_sec),
            format_rate(s.network.download_bytes_per_sec)
        ),
        &app.history.network,
        pulse_theme::NETWORK,
    );
    let system = detail_card(
        "System",
        vec![
            format!("{} • {}", app.system.hostname, app.system.os_version),
            format!(
                "Build {} • user {}",
                app.system.build_number, app.system.username
            ),
            format!("Uptime {}", format_uptime(s.uptime_seconds)),
            format!("Processes {}", s.processes.len()),
            format!("Updated {}", app.last_update),
        ],
    );
    let warning_label = if s.warnings.is_empty() {
        "All active collectors reporting.".to_string()
    } else {
        format!("Collector notes: {}", s.warnings.join(" | "))
    };
    let warning = text(warning_label)
        .size(12)
        .style(move |theme: &Theme| text::Style {
            color: Some(if s.warnings.is_empty() {
                pulse_theme::text_secondary(theme)
            } else {
                pulse_theme::HIGHLIGHT
            }),
        });
    let settings = button(
        text("Open Settings")
            .size(13)
            .style(|theme: &Theme| text::Style {
                color: Some(pulse_theme::text_secondary(theme)),
            }),
    )
    .on_press(Message::Navigate(Page::Settings))
    .style(pulse_theme::subtle_button_style());
    scrollable(
        column![
            section_title("Overview"),
            row![cpu, memory].spacing(14),
            row![gpu, network].spacing(14),
            row![disk, system].spacing(14),
            warning,
            settings,
        ]
        .spacing(14)
        .padding(20),
    )
    .height(Length::Fill)
    .into()
}
