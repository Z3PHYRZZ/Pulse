use crate::app::PulseApp;
use crate::config::ThemeMode;
use crate::messages::Message;
use crate::ui::theme as pulse_theme;
use crate::ui::widgets::section_title;
use iced::widget::{button, checkbox, column, row, scrollable, slider, text};
use iced::{Element, Theme};

pub fn view(app: &PulseApp) -> Element<'_, Message> {
    let cfg = app.config.get();
    let mut draft = cfg.clone();
    draft.refresh_ms = app.pending_refresh_ms.unwrap_or(cfg.refresh_ms);
    draft.history_seconds = app.pending_history_seconds.unwrap_or(cfg.history_seconds);
    draft.enable_startup = app.pending_startup.unwrap_or(cfg.enable_startup);
    draft.minimize_to_tray = app.pending_tray.unwrap_or(cfg.minimize_to_tray);
    let active_theme = app.pending_theme.unwrap_or_else(|| draft.theme_mode());
    let refresh_slider = slider(250.0f32..=5000.0f32, draft.refresh_ms as f32, |v: f32| {
        Message::RefreshChanged(v.round() as u64)
    })
    .step(250.0f32);
    let history_slider = slider(
        30.0f32..=300.0f32,
        draft.history_seconds as f32,
        |v: f32| Message::HistoryChanged(v.round() as usize),
    )
    .step(30.0f32);

    let dark_button = theme_button("Dark", ThemeMode::Dark, active_theme);
    let light_button = theme_button("Light", ThemeMode::Light, active_theme);
    let save_button = button(
        text("Save settings")
            .size(13)
            .style(|theme: &Theme| text::Style {
                color: Some(pulse_theme::button_text(theme)),
            }),
    )
    .on_press(Message::SaveSettings(draft.clone()))
    .style(pulse_theme::primary_button_style(true));
    let folder_button =
        button(
            text("Open config folder")
                .size(13)
                .style(|theme: &Theme| text::Style {
                    color: Some(pulse_theme::text_secondary(theme)),
                }),
        )
        .on_press(Message::OpenConfigFolder)
        .style(pulse_theme::subtle_button_style());

    scrollable(
        column![
            section_title("Settings"),
            text(format!("Refresh interval: {} ms", draft.refresh_ms)).size(15),
            refresh_slider,
            text(format!("Graph history: {} seconds", draft.history_seconds)).size(15),
            history_slider,
            text("Appearance").size(16),
            row![dark_button, light_button].spacing(10),
            checkbox("Launch Pulse at Windows startup", draft.enable_startup)
                .on_toggle(Message::ToggleStartup),
            checkbox("Minimize to system tray", draft.minimize_to_tray)
                .on_toggle(Message::ToggleMinimizeToTray),
            row![save_button, folder_button].spacing(10),
            text(format!("Config file: {}", app.config_path.display()))
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(pulse_theme::text_secondary(theme)),
                }),
            text("Startup writes HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run.")
                .size(12)
                .style(|theme: &Theme| text::Style {
                    color: Some(pulse_theme::text_secondary(theme)),
                }),
            text("Close button exits fully. Use Hide to tray to keep Pulse in the background.")
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

fn theme_button<'a>(
    label: &'a str,
    mode: ThemeMode,
    active: ThemeMode,
) -> iced::widget::Button<'a, Message> {
    let selected = mode == active;
    button(
        text(label)
            .size(13)
            .style(move |theme: &Theme| text::Style {
                color: Some(if selected {
                    pulse_theme::button_text(theme)
                } else {
                    pulse_theme::text_secondary(theme)
                }),
            }),
    )
    .on_press(Message::ThemeChanged(mode))
    .style(pulse_theme::primary_button_style(selected))
}
