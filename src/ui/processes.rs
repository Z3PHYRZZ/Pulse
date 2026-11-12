use crate::app::PulseApp;
use crate::messages::Message;
use crate::models::ProcessSort;
use crate::ui::widgets::section_title;
use crate::utils::format_bytes;
use iced::widget::{column, row, scrollable, text, text_input};
use iced::{Element, Length};

pub fn view(app: &PulseApp) -> Element<'_, Message> {
    let mut rows = column![
        row![
            sort_button("CPU", ProcessSort::Cpu, app.sort),
            sort_button("Memory", ProcessSort::Memory, app.sort),
            sort_button("Name", ProcessSort::Name, app.sort),
            sort_button("PID", ProcessSort::Pid, app.sort),
        ]
        .spacing(8),
        text_input("filter processes", &app.filter)
            .on_input(Message::FilterChanged)
            .width(240),
        row![
            text("PID").width(72).size(13),
            text("Process").width(Length::Fill).size(13),
            text("CPU").width(90).size(13),
            text("RAM").width(110).size(13),
        ]
        .spacing(8),
    ]
    .spacing(10);
    for process in app.visible_processes() {
        rows = rows.push(
            row![
                text(process.pid.to_string()).width(72).size(13),
                text(process.name.clone()).width(Length::Fill).size(13),
                text(format!("{:.1}%", process.cpu_percent))
                    .width(90)
                    .size(13),
                text(format_bytes(process.memory_bytes)).width(110).size(13),
            ]
            .spacing(8),
        );
    }
    scrollable(
        column![
            section_title("Processes"),
            text(format!(
                "{} shown of {}",
                app.visible_processes().count(),
                app.latest.processes.len()
            ))
            .size(12),
            rows,
        ]
        .spacing(14)
        .padding(20),
    )
    .into()
}

fn sort_button<'a>(
    label: &'a str,
    value: ProcessSort,
    active: ProcessSort,
) -> iced::widget::Button<'a, Message> {
    let button = iced::widget::button(text(label).size(12)).on_press(Message::SortChanged(value));
    if value == active {
        button.style(iced::widget::button::primary)
    } else {
        button
    }
}
