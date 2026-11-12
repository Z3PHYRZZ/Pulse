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
    color: Color,
) -> Element<'a, Message> {
    let title = title.to_string();
    let value = value.to_string();
    let detail = detail.to_string();
    let graph = canvas_widget(Sparkline::new(history, color))
        .width(Length::Fill)
        .height(64);
    container(
        column![
            text(title).size(14),
            text(value).size(30),
            text(detail).size(12),
            graph,
        ]
        .spacing(6),
    )
    .padding(16)
    .style(move |theme: &Theme| container::Style {
        background: Some(iced::Background::Color(
            theme.palette().background.scale_alpha(0.85),
        )),
        border: iced::Border {
            color: color.scale_alpha(0.45),
            width: 1.0,
            radius: 10.0.into(),
        },
        ..container::Style::default()
    })
    .width(Length::Fill)
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
        let values: Vec<f32> = self.history.values().collect();
        if values.len() < 2 {
            return vec![frame.into_geometry()];
        }
        let width = bounds.width.max(1.0);
        let height = bounds.height.max(1.0);
        let step = width / (values.len() - 1) as f32;
        let line = Path::new(|builder| {
            for (index, value) in values.iter().enumerate() {
                let x = index as f32 * step;
                let y = height - value / 100.0 * height;
                if index == 0 {
                    builder.move_to(iced::Point::new(x, y));
                } else {
                    builder.line_to(iced::Point::new(x, y));
                }
            }
        });
        frame.stroke(
            &line,
            Stroke::default().with_width(2.0).with_color(self.color),
        );
        let area = Path::new(|builder| {
            builder.move_to(iced::Point::new(0.0, height));
            for (index, value) in values.iter().enumerate() {
                builder.line_to(iced::Point::new(
                    index as f32 * step,
                    height - value / 100.0 * height,
                ));
            }
            builder.line_to(iced::Point::new(width, height));
            builder.close();
        });
        frame.fill(&area, self.color.scale_alpha(0.12));
        vec![frame.into_geometry()]
    }
}
