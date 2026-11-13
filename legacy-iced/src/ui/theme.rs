// language: Rust, file: theme.rs, target: Windows 11 x86_64, edition 2021
use iced::widget::{button, container};
use iced::{color, Border, Color, Theme};

pub const BACKGROUND: Color = color!(0x0D1821);
pub const SIDEBAR: Color = color!(0x142333);
pub const CARD: Color = color!(0x344966);
pub const TEXT_PRIMARY: Color = color!(0xF0F4EF);
pub const TEXT_SECONDARY: Color = color!(0xB4CDED);
pub const BUTTON_PRIMARY: Color = color!(0xB4CDED);
pub const BUTTON_TEXT: Color = color!(0x0D1821);
pub const HIGHLIGHT: Color = color!(0xB86F52);
pub const BORDER: Color = color!(0x52677E);

pub const CPU: Color = color!(0x81A9CD);
pub const MEMORY: Color = color!(0xA9C7E8);
pub const GPU: Color = color!(0xAFA0D1);
pub const DISK: Color = color!(0xBFA77E);
pub const NETWORK: Color = color!(0x83B4AE);

pub const CORNER_RADIUS: f32 = 10.0;
pub const BORDER_WIDTH: f32 = 1.0;
pub const GRAPH_WIDTH: f32 = 1.35;
pub const HIGH_TEMP_C: f32 = 80.0;

pub fn dark() -> Theme {
    Theme::custom(
        "Pulse Dark".to_string(),
        iced::theme::Palette {
            background: BACKGROUND,
            text: TEXT_PRIMARY,
            primary: BUTTON_PRIMARY,
            success: NETWORK,
            danger: HIGHLIGHT,
        },
    )
}

pub fn light() -> Theme {
    let background = color!(0xF0F4EF);
    let text = color!(0x0D1821);
    Theme::custom(
        "Pulse Light".to_string(),
        iced::theme::Palette {
            background,
            text,
            primary: color!(0x344966),
            success: color!(0x3E776F),
            danger: HIGHLIGHT,
        },
    )
}

pub fn is_light(theme: &Theme) -> bool {
    theme.palette().text == color!(0x0D1821)
}

pub fn surface(theme: &Theme) -> Color {
    if is_light(theme) {
        color!(0xF0F4EF)
    } else {
        BACKGROUND
    }
}

pub fn sidebar(theme: &Theme) -> Color {
    if is_light(theme) {
        color!(0xE1E9E6)
    } else {
        SIDEBAR
    }
}

pub fn card(theme: &Theme) -> Color {
    if is_light(theme) {
        color!(0xCEDCE4)
    } else {
        CARD
    }
}

pub fn text_primary(theme: &Theme) -> Color {
    theme.palette().text
}

pub fn text_secondary(theme: &Theme) -> Color {
    if is_light(theme) {
        color!(0x344966)
    } else {
        TEXT_SECONDARY
    }
}

pub fn border(theme: &Theme) -> Color {
    if is_light(theme) {
        color!(0x8FA3AF)
    } else {
        BORDER
    }
}

pub fn button_primary(theme: &Theme) -> Color {
    if is_light(theme) {
        color!(0x344966)
    } else {
        BUTTON_PRIMARY
    }
}

pub fn button_text(theme: &Theme) -> Color {
    if is_light(theme) {
        color!(0xF0F4EF)
    } else {
        BUTTON_TEXT
    }
}

pub fn card_style(theme: &Theme, accent: Color) -> container::Style {
    container::Style {
        text_color: Some(text_primary(theme)),
        background: Some(card(theme).scale_alpha(0.72).into()),
        border: thin_border(theme, accent),
        ..container::Style::default()
    }
}

pub fn system_card_style(theme: &Theme) -> container::Style {
    card_style(theme, border(theme))
}

pub fn sidebar_style(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(text_primary(theme)),
        background: Some(sidebar(theme).into()),
        border: Border {
            color: border(theme).scale_alpha(0.32),
            width: BORDER_WIDTH,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn top_bar_style(theme: &Theme) -> container::Style {
    container::Style {
        text_color: Some(text_primary(theme)),
        background: Some(sidebar(theme).scale_alpha(0.88).into()),
        border: Border {
            color: border(theme).scale_alpha(0.30),
            width: BORDER_WIDTH,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    }
}

pub fn thin_border(theme: &Theme, color: Color) -> Border {
    Border {
        color: color.scale_alpha(if is_light(theme) { 0.70 } else { 0.62 }),
        width: BORDER_WIDTH,
        radius: CORNER_RADIUS.into(),
    }
}

pub fn nav_button_style(
    active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style + 'static {
    move |theme, status| {
        let base = button::Style {
            text_color: if active {
                text_primary(theme)
            } else {
                text_secondary(theme)
            },
            background: if active {
                Some(card(theme).scale_alpha(0.62).into())
            } else {
                None
            },
            border: Border {
                color: if active {
                    text_secondary(theme).scale_alpha(0.44)
                } else {
                    Color::TRANSPARENT
                },
                width: BORDER_WIDTH,
                radius: CORNER_RADIUS.into(),
            },
            ..button::Style::default()
        };
        match status {
            button::Status::Hovered => button::Style {
                background: Some(card(theme).scale_alpha(0.48).into()),
                border: Border {
                    color: border(theme).scale_alpha(0.48),
                    ..base.border
                },
                ..base
            },
            button::Status::Disabled => button::Style {
                text_color: text_secondary(theme).scale_alpha(0.5),
                ..base
            },
            button::Status::Active | button::Status::Pressed => base,
        }
    }
}

pub fn primary_button_style(
    active: bool,
) -> impl Fn(&Theme, button::Status) -> button::Style + 'static {
    move |theme, status| {
        let base = button::Style {
            text_color: if active {
                button_text(theme)
            } else {
                text_secondary(theme)
            },
            background: if active {
                Some(button_primary(theme).into())
            } else {
                None
            },
            border: thin_border(
                theme,
                if active {
                    button_primary(theme)
                } else {
                    border(theme)
                },
            ),
            ..button::Style::default()
        };
        match status {
            button::Status::Hovered => button::Style {
                background: Some(button_primary(theme).scale_alpha(0.82).into()),
                text_color: button_text(theme),
                ..base
            },
            button::Status::Disabled => button::Style {
                text_color: text_secondary(theme).scale_alpha(0.5),
                ..base
            },
            button::Status::Active | button::Status::Pressed => base,
        }
    }
}

pub fn subtle_button_style() -> impl Fn(&Theme, button::Status) -> button::Style + 'static {
    move |theme, status| {
        let base = button::Style {
            text_color: text_secondary(theme),
            background: None,
            border: thin_border(theme, border(theme)),
            ..button::Style::default()
        };
        match status {
            button::Status::Hovered => button::Style {
                text_color: text_primary(theme),
                background: Some(card(theme).scale_alpha(0.42).into()),
                ..base
            },
            button::Status::Disabled => button::Style {
                text_color: text_secondary(theme).scale_alpha(0.5),
                ..base
            },
            button::Status::Active | button::Status::Pressed => base,
        }
    }
}

pub fn warning_color(theme: &Theme, value: Option<f32>) -> Color {
    if value.is_some_and(|temperature| temperature >= HIGH_TEMP_C) {
        HIGHLIGHT
    } else {
        text_secondary(theme)
    }
}
