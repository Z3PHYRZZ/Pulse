use crate::app::PulseApp;
use crate::messages::Message;
use crate::ui::widgets::{metric_card, section_title};
use crate::utils::{format_bytes, percent_text};
use iced::widget::{column, scrollable, text};
use iced::{Color, Element};

pub fn view(app: &PulseApp) -> Element<'_, Message> {
    let g = &app.latest.gpu;
    let usage = g
        .usage_percent
        .map(percent_text)
        .unwrap_or_else(|| "n/a".to_string());
    let vram = match (g.vram_used_bytes, g.vram_total_bytes) {
        (Some(used), Some(total)) => format!("{} of {}", format_bytes(used), format_bytes(total)),
        _ => "unavailable".to_string(),
    };
    let temp = g
        .temperature_c
        .map(|v| format!("{v:.0} °C"))
        .unwrap_or_else(|| "unavailable".to_string());
    scrollable(
        column![
            section_title("GPU"),
            metric_card("Utilization", usage, format!("{} • {}", g.name, g.source), &app.history.gpu, Color::from_rgb8(0xB7, 0x7C, 0xFF)),
            text(format!("VRAM: {vram}")).size(15),
            text(format!("Temperature: {temp}")).size(15),
            text("NVML supplies NVIDIA utilization, VRAM and temperature. DXGI supplies adapter identity and dedicated VRAM only.").size(12),
        ]
        .spacing(14)
        .padding(20),
    )
    .into()
}
