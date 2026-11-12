use crate::app::PulseApp;
use crate::messages::{Message, Page};
use crate::ui::widgets::{metric_card, section_title};
use crate::utils::{format_bytes, format_rate, format_uptime, percent_text};
use iced::widget::{column, container, row, scrollable, text};
use iced::{Color, Element, Length};

pub fn view(app: &PulseApp) -> Element<'_, Message> {
    let s = &app.latest;
    let cpu = metric_card(
        "CPU",
        percent_text(s.cpu.usage_percent),
        format!("{} logical cores • {}", s.cpu.logical_cores, s.cpu.name),
        &app.history.cpu,
        Color::from_rgb8(0x4C, 0xD9, 0x64),
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
        Color::from_rgb8(0x5A, 0x9C, 0xFF),
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
        Color::from_rgb8(0xB7, 0x7C, 0xFF),
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
        Color::from_rgb8(0xFF, 0xC1, 0x4D),
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
        Color::from_rgb8(0x35, 0xD0, 0xC0),
    );
    let system = container(
        column![
            text("System").size(18),
            text(format!(
                "{} • {}",
                app.system.hostname, app.system.os_version
            ))
            .size(14),
            text(format!(
                "Build {} • user {}",
                app.system.build_number, app.system.username
            ))
            .size(13),
            text(format!("Uptime {}", format_uptime(s.uptime_seconds))).size(13),
            text(format!("Processes {}", s.processes.len())).size(13),
            text(format!("Updated {}", app.last_update)).size(12),
        ]
        .spacing(8),
    )
    .padding(16)
    .width(Length::Fill)
    .style(|theme: &iced::Theme| container::Style {
        background: Some(iced::Background::Color(
            theme.palette().background.scale_alpha(0.85),
        )),
        border: iced::Border {
            color: theme.palette().text.scale_alpha(0.12),
            width: 1.0,
            radius: 10.0.into(),
        },
        ..container::Style::default()
    });
    let warning = if s.warnings.is_empty() {
        text("All active collectors reporting.").size(12)
    } else {
        text(format!("Collector notes: {}", s.warnings.join(" | "))).size(12)
    };
    scrollable(
        column![
            section_title("Overview"),
            row![cpu, memory].spacing(14),
            row![gpu, network].spacing(14),
            row![disk, system].spacing(14),
            warning,
            iced::widget::button(text("Open Settings").size(13))
                .on_press(Message::Navigate(Page::Settings)),
        ]
        .spacing(14)
        .padding(20),
    )
    .height(Length::Fill)
    .into()
}
