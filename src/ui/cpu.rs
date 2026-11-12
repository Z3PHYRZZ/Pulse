use crate::app::PulseApp;
use crate::messages::Message;
use crate::ui::widgets::{metric_card, section_title};
use crate::utils::percent_text;
use iced::widget::{column, scrollable, text};
use iced::{Color, Element};

pub fn view(app: &PulseApp) -> Element<'_, Message> {
    let cpu = &app.latest.cpu;
    let freq = cpu
        .frequency_mhz
        .map(|v| format!("{v:.0} MHz"))
        .unwrap_or_else(|| "unavailable".to_string());
    let temp = cpu
        .temperature_c
        .map(|v| format!("{v:.0} °C"))
        .unwrap_or_else(|| "driver support required".to_string());
    let mut core_rows =
        column![text(format!("Logical cores: {}", cpu.logical_cores)).size(15)].spacing(6);
    for (index, usage) in cpu.per_core_percent.iter().enumerate() {
        core_rows =
            core_rows.push(text(format!("Core {index:02}: {}", percent_text(*usage))).size(13));
    }
    scrollable(
        column![
            section_title("CPU"),
            metric_card(
                "Utilization",
                percent_text(cpu.usage_percent),
                cpu.name.clone(),
                &app.history.cpu,
                Color::from_rgb8(0x4C, 0xD9, 0x64),
            ),
            text(format!("Frequency: {freq}")).size(15),
            text(format!("Temperature: {temp}")).size(15),
            text("Temperature is optional; Windows standard performance APIs do not guarantee a sensor path.")
                .size(12),
            core_rows,
        ]
        .spacing(14)
        .padding(20),
    )
    .into()
}
