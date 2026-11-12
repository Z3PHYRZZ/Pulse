use crate::app::PulseApp;
use crate::messages::Message;
use crate::ui::widgets::{metric_card, section_title};
use crate::utils::{format_bytes, percent_text};
use iced::widget::{column, scrollable, text};
use iced::{Color, Element};

pub fn view(app: &PulseApp) -> Element<'_, Message> {
    let m = &app.latest.memory;
    scrollable(
        column![
            section_title("Memory"),
            metric_card(
                "Physical memory",
                percent_text(m.usage_percent),
                format!("{} total", format_bytes(m.total_bytes)),
                &app.history.memory,
                Color::from_rgb8(0x5A, 0x9C, 0xFF)
            ),
            text(format!("Used: {}", format_bytes(m.used_bytes))).size(15),
            text(format!("Available: {}", format_bytes(m.available_bytes))).size(15),
            text(format!("Total: {}", format_bytes(m.total_bytes))).size(15),
        ]
        .spacing(14)
        .padding(20),
    )
    .into()
}
