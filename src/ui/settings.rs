use crate::app::PulseApp;
use crate::config::ThemeMode;
use crate::messages::Message;
use crate::ui::widgets::section_title;
use iced::widget::{button, checkbox, column, row, scrollable, slider, text};
use iced::Element;

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

    let dark_button =
        button(text("Dark").size(13)).on_press(Message::ThemeChanged(ThemeMode::Dark));
    let light_button =
        button(text("Light").size(13)).on_press(Message::ThemeChanged(ThemeMode::Light));

    scrollable(
        column![
            section_title("Settings"),
            text(format!("Refresh interval: {} ms", draft.refresh_ms)).size(15),
            refresh_slider,
            text(format!("Graph history: {} seconds", draft.history_seconds)).size(15),
            history_slider,
            text("Appearance").size(16),
            row![
                if active_theme == ThemeMode::Dark {
                    dark_button.style(iced::widget::button::primary)
                } else {
                    dark_button
                },
                if active_theme == ThemeMode::Light {
                    light_button.style(iced::widget::button::primary)
                } else {
                    light_button
                },
            ]
            .spacing(10),
            checkbox("Launch Pulse at Windows startup", draft.enable_startup)
                .on_toggle(Message::ToggleStartup),
            checkbox("Minimize to system tray", draft.minimize_to_tray)
                .on_toggle(Message::ToggleMinimizeToTray),
            row![
                button(text("Save settings").size(13)).on_press(Message::SaveSettings(draft)),
                button(text("Open config folder").size(13)).on_press(Message::OpenConfigFolder),
            ]
            .spacing(10),
            text(format!("Config file: {}", app.config_path.display())).size(12),
            text("Startup writes HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run.")
                .size(12),
            text("Close button exits fully. Use Hide to tray to keep Pulse in the background.")
                .size(12),
        ]
        .spacing(14)
        .padding(20),
    )
    .into()
}
