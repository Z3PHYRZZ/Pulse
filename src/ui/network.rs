use crate::app::PulseApp;
use crate::messages::Message;
use crate::ui::theme as pulse_theme;
use crate::ui::widgets::{metric_card, section_title};
use crate::utils::format_rate;
use iced::widget::{column, scrollable, text};
use iced::Element;

pub fn view(app: &PulseApp) -> Element<'_, Message> {
    let n = &app.latest.network;
    let mut rows = column![
        metric_card(
            "Total throughput",
            format_rate(n.upload_bytes_per_sec + n.download_bytes_per_sec),
            "Bytes/sec across network interfaces",
            &app.history.network,
            pulse_theme::NETWORK
        ),
        text(format!("Upload: {}", format_rate(n.upload_bytes_per_sec))).size(15),
        text(format!(
            "Download: {}",
            format_rate(n.download_bytes_per_sec)
        ))
        .size(15),
    ]
    .spacing(10);
    for adapter in &n.adapters {
        rows = rows.push(
            text(format!(
                "{} — ↑ {} ↓ {}",
                adapter.name,
                format_rate(adapter.upload_bytes_per_sec),
                format_rate(adapter.download_bytes_per_sec)
            ))
            .size(13),
        );
    }
    scrollable(
        column![section_title("Network"), rows]
            .spacing(14)
            .padding(20),
    )
    .into()
}
