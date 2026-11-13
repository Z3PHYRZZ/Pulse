// language: Rust, file: widgets.rs, target: Windows 11 x86_64, edition 2021
use crate::ui::theme;
use crate::utils::History;
use iced::widget::canvas::{self, Geometry, Path, Stroke};
use iced::widget::{canvas as canvas_widget, column, container, text, Column};
use iced::{Color, Element, Length, Rectangle, Renderer, Theme};

pub fn section_title<'a, Message: 'a>(value: impl ToString) -> Column<'a, Message> {
    column![
        text(value.to_string()).size(22),
        iced::widget::horizontal_rule(1),
    ]
    .spacing(8)
}

pub fn metric_card<'a, Message: 'a>(
    title: impl ToString,
    value: impl ToString,
    detail: impl ToString,
    history: &'a History,
    accent: Color,
) -> Element<'a, Message> {
    let title = title.to_string();
    let value = value.to_string();
    let detail = detail.to_string();
    let graph = canvas_widget(Sparkline::new(history, accent))
        .width(Length::Fill)
        .height(56);
    container(
        column![
            text(title)
                .size(13)
                .style(move |current: &Theme| text::Style {
                    color: Some(theme::text_secondary(current)),
                }),
            text(value)
                .size(28)
                .style(move |current: &Theme| text::Style {
                    color: Some(theme::text_primary(current)),
                }),
            text(detail)
                .size(11)
                .style(move |current: &Theme| text::Style {
                    color: Some(theme::text_secondary(current)),
                }),
            graph,
        ]
        .spacing(5),
    )
    .padding(14)
    .style(move |current: &Theme| theme::card_style(current, accent))
    .width(Length::Fill)
    .into()
}

pub fn detail_card<'a, Message: 'a>(
    title: impl ToString,
    rows: Vec<String>,
) -> Element<'a, Message> {
    let title = title.to_string();
    let mut column = column![text(title).size(16)].spacing(6);
    for row in rows {
        column = column.push(text(row).size(13).style(|current: &Theme| text::Style {
            color: Some(theme::text_secondary(current)),
        }));
    }
    container(column)
        .padding(14)
        .width(Length::Fill)
        .style(|current: &Theme| theme::system_card_style(current))
        .into()
}

pub struct Sparkline<'a> {
    history: &'a History,
    color: Color,
}

impl<'a> Sparkline<'a> {
    pub fn new(history: &'a History, color: Color) -> Self {
        Self { history, color }
    }
}

impl<Message> canvas::Program<Message> for Sparkline<'_> {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let raw: Vec<f32> = self.history.values().collect();
        if raw.len() < 2 {
            return vec![frame.into_geometry()];
        }
        let width = bounds.width.max(1.0);
        let height = bounds.height.max(1.0);
        let padding = 3.0;
        let usable_height = (height - padding * 2.0).max(1.0);
        let step = width / (raw.len() - 1) as f32;
        let mut values = smooth_values(&raw);
        for value in &mut values {
            *value = padding + (1.0 - value.clamp(0.0, 100.0) / 100.0) * usable_height;
        }
        let line = smooth_path(&values, step);
        frame.stroke(
            &line,
            Stroke::default()
                .with_width(theme::GRAPH_WIDTH)
                .with_color(self.color),
        );
        let area = Path::new(|builder| {
            builder.move_to(iced::Point::new(0.0, height));
            for (index, value) in values.iter().enumerate() {
                builder.line_to(iced::Point::new(index as f32 * step, *value));
            }
            builder.line_to(iced::Point::new(width, height));
            builder.close();
        });
        frame.fill(&area, self.color.scale_alpha(0.07));
        vec![frame.into_geometry()]
    }
}

fn smooth_values(values: &[f32]) -> Vec<f32> {
    let mut smoothed = Vec::with_capacity(values.len());
    for index in 0..values.len() {
        let previous = values[index.saturating_sub(1)];
        let current = values[index];
        let next = values[(index + 1).min(values.len() - 1)];
        smoothed.push(previous * 0.22 + current * 0.56 + next * 0.22);
    }
    smoothed
}

fn smooth_path(values: &[f32], step: f32) -> Path {
    Path::new(move |builder| {
        builder.move_to(iced::Point::new(0.0, values[0]));
        for index in 1..values.len() {
            let previous_x = (index - 1) as f32 * step;
            let previous_y = values[index - 1];
            let x = index as f32 * step;
            let y = values[index];
            let middle_x = (previous_x + x) * 0.5;
            builder.quadratic_curve_to(
                iced::Point::new(previous_x, previous_y),
                iced::Point::new(middle_x, (previous_y + y) * 0.5),
            );
            builder.quadratic_curve_to(iced::Point::new(x, y), iced::Point::new(x, y));
        }
    })
}
