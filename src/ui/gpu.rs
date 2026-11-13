use crate::app::PulseApp;
use crate::messages::Message;
use crate::ui::theme as pulse_theme;
use crate::ui::widgets::{metric_card, section_title};
use crate::utils::{format_bytes, percent_text};
use iced::widget::{column, scrollable, text};
use iced::{Element, Theme};

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
    let temperature = text(format!("Temperature: {temp}"))
        .size(15)
        .style(move |theme: &Theme| text::Style {
            color: Some(pulse_theme::warning_color(theme, g.temperature_c)),
        });
    scrollable(
        column![
            section_title("GPU"),
            metric_card(
                "Utilization",
                usage,
                format!("{} • {}", g.name, g.source),
                &app.history.gpu,
                pulse_theme::GPU,
            ),
            text(format!("VRAM: {vram}")).size(15),
            temperature,
            text("NVML supplies NVIDIA utilization, VRAM and temperature. DXGI supplies adapter identity and dedicated VRAM only.")
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(pulse_theme::text_secondary(theme)),
                }),
        ]
        .spacing(14)
        .padding(20),
    )
    .into()
}
