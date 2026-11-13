use crate::config::{Config, ThemeMode};
use crate::models::{MetricSample, ProcessSort};
use std::sync::mpsc::Receiver;

#[derive(Debug, Clone)]
pub enum Message {
    Metrics(MetricSample),
    Navigate(Page),
    SortChanged(ProcessSort),
    FilterChanged(String),
    RefreshChanged(u64),
    HistoryChanged(usize),
    ToggleStartup(bool),
    ToggleMinimizeToTray(bool),
    ThemeChanged(ThemeMode),
    SaveSettings(Config),
    HideWindow,

    WindowDiscovered(Option<iced::window::Id>),
    MinimizeDiscovered(bool, Option<iced::window::Id>),

    Exit,
    OpenConfigFolder,
    TrayEvent(TrayEvent),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Page {
    #[default]
    Overview,
    Cpu,
    Memory,
    Gpu,
    Storage,
    Network,
    Processes,
    Settings,
}

impl Page {
    pub const ALL: [Page; 8] = [
        Page::Overview,
        Page::Cpu,
        Page::Memory,
        Page::Gpu,
        Page::Storage,
        Page::Network,
        Page::Processes,
        Page::Settings,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Page::Overview => "Overview",
            Page::Cpu => "CPU",
            Page::Memory => "Memory",
            Page::Gpu => "GPU",
            Page::Storage => "Storage",
            Page::Network => "Network",
            Page::Processes => "Processes",
            Page::Settings => "Settings",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayEvent {
    Show,
    Hide,
    Exit,
}

pub fn drain_tray(receiver: &Receiver<TrayEvent>) -> Option<TrayEvent> {
    let mut last = None;
    while let Ok(event) = receiver.try_recv() {
        last = Some(event);
    }
    last
}
