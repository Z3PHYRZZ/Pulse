use crate::app::PulseApp;
use crate::messages::Message;
use crate::ui::widgets::{metric_card, section_title};
use crate::utils::{format_bytes, format_rate, percent_text};
use iced::widget::{column, scrollable, text};
use iced::{Color, Element};

pub fn view(app: &PulseApp) -> Element<'_, Message> {
    let d = &app.latest.disk;
    let mut rows = column![
        metric_card(
            "Disk throughput",
            format_rate(d.read_bytes_per_sec + d.write_bytes_per_sec),
            format!(
                "R {} • W {}",
                format_rate(d.read_bytes_per_sec),
                format_rate(d.write_bytes_per_sec)
            ),
            &app.history.disk,
            Color::from_rgb8(0xFF, 0xC1, 0x4D)
        ),
        text("Volumes").size(18),
    ]
    .spacing(10);
    for volume in &d.volumes {
        rows = rows.push(
            text(format!(
                "{} — {} / {} ({})",
                volume.name,
                format_bytes(volume.used_bytes),
                format_bytes(volume.total_bytes),
                percent_text(volume.usage_percent)
            ))
            .size(14),
        );
    }
    scrollable(
        column![section_title("Storage"), rows]
            .spacing(14)
            .padding(20),
    )
    .into()
}
