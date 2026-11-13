use crate::config::{Config, ConfigState, ThemeMode};
use crate::engine::EngineHandle;
use crate::messages::{Message, Page, TrayEvent};
use crate::models::{MetricSample, ProcessInfo, ProcessSort, SystemInfo};
use crate::ui;
use crate::utils::{format_rate, History};
use anyhow::Result;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length, Task, Theme};
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

pub struct PulseApp {
    pub engine: EngineHandle,
    pub system: Arc<SystemInfo>,
    pub config: Arc<ConfigState>,
    pub config_path: PathBuf,
    pub latest: MetricSample,
    pub page: Page,
    pub sort: ProcessSort,
    pub filter: String,
    pub history: AppHistory,
    pub visible: bool,
    pub pending_refresh_ms: Option<u64>,
    pub pending_history_seconds: Option<usize>,
    pub pending_startup: Option<bool>,
    pub pending_tray: Option<bool>,
    pub pending_theme: Option<ThemeMode>,
    pub last_update: String,
    pub status: String,
    pub window_id: Option<iced::window::Id>,
    pub tray_receiver: Arc<Mutex<Receiver<TrayEvent>>>,
}

#[derive(Debug, Clone)]
pub struct AppHistory {
    pub cpu: History,
    pub memory: History,
    pub gpu: History,
    pub disk: History,
    pub network: History,
}

impl AppHistory {
    fn from_config(config: &Config) -> Self {
        let len = config.history_len();
        Self {
            cpu: History::new(len),
            memory: History::new(len),
            gpu: History::new(len),
            disk: History::new(len),
            network: History::new(len),
        }
    }

    fn resize(&mut self, len: usize) {
        self.cpu.resize(len);
        self.memory.resize(len);
        self.gpu.resize(len);
        self.disk.resize(len);
        self.network.resize(len);
    }
}

impl PulseApp {
    pub fn new(
        engine: EngineHandle,
        config_path: PathBuf,
        tray_receiver: Receiver<TrayEvent>,
    ) -> Result<Self> {
        let config = engine.config.clone();
        let system = engine.system.clone();
        let initial = config.get();
        let startup = crate::windows::startup::is_enabled();
        Ok(Self {
            engine,
            system,
            config,
            config_path,
            latest: MetricSample::default(),
            page: Page::Overview,
            sort: initial.sort_key(),
            filter: String::new(),
            history: AppHistory::from_config(&initial),
            visible: !initial.start_minimized,
            pending_refresh_ms: None,
            pending_history_seconds: None,
            pending_startup: None,
            pending_tray: None,
            pending_theme: None,
            last_update: "waiting for first sample".to_string(),
            window_id: None,
            status: if startup {
                "startup integration active".to_string()
            } else {
                "ready".to_string()
            },
            tray_receiver: Arc::new(Mutex::new(tray_receiver)),
        })
    }

    pub fn title(&self) -> String {
        format!("Pulse — {}", self.page.label())
    }

    pub fn theme(&self) -> Theme {
        self.pending_theme
            .unwrap_or_else(|| self.config.get().theme_mode())
            .to_iced()
    }

    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Metrics(sample) => {
                self.history.cpu.push(sample.cpu.usage_percent);
                self.history.memory.push(sample.memory.usage_percent);
                self.history
                    .gpu
                    .push(sample.gpu.usage_percent.unwrap_or(0.0));
                self.history.disk.push(rate_to_percent(
                    sample.disk.read_bytes_per_sec + sample.disk.write_bytes_per_sec,
                ));
                self.history.network.push(rate_to_percent(
                    sample.network.upload_bytes_per_sec + sample.network.download_bytes_per_sec,
                ));
                self.latest = sample;
                self.last_update = chrono::Local::now().format("%H:%M:%S").to_string();
                Task::none()
            }
            Message::Navigate(page) => {
                self.page = page;
                Task::none()
            }
            Message::SortChanged(sort) => {
                self.sort = sort;
                Task::none()
            }
            Message::FilterChanged(filter) => {
                self.filter = filter;
                Task::none()
            }
            Message::RefreshChanged(ms) => {
                self.pending_refresh_ms = Some(ms);
                Task::none()
            }
            Message::HistoryChanged(seconds) => {
                self.pending_history_seconds = Some(seconds);
                Task::none()
            }
            Message::ToggleStartup(value) => {
                self.pending_startup = Some(value);
                Task::none()
            }
            Message::ToggleMinimizeToTray(value) => {
                self.pending_tray = Some(value);
                Task::none()
            }
            Message::ThemeChanged(mode) => {
                self.pending_theme = Some(mode);
                let mut draft = self.config.get();
                draft.theme = mode.as_str().to_string();
                match self.config.set(draft) {
                    Ok(()) => {
                        self.pending_theme = None;
                        self.status = format!("{} mode active", mode.label());
                    }
                    Err(err) => self.status = format!("theme save failed: {err:#}"),
                }
                Task::none()
            }
            Message::SaveSettings(mut draft) => {
                draft.process_sort = sort_name(self.sort);
                draft.theme = self
                    .pending_theme
                    .unwrap_or_else(|| draft.theme_mode())
                    .as_str()
                    .to_string();
                let startup = draft.enable_startup;
                match self
                    .config
                    .set(draft.clone())
                    .and_then(|_| apply_startup(startup))
                {
                    Ok(()) => {
                        self.history.resize(draft.history_len());
                        self.pending_refresh_ms = None;
                        self.pending_history_seconds = None;
                        self.pending_startup = None;
                        self.pending_tray = None;
                        self.pending_theme = None;
                        self.status = format!(
                            "settings saved at {}",
                            chrono::Local::now().format("%H:%M:%S")
                        );
                    }
                    Err(err) => self.status = format!("save failed: {err:#}"),
                }
                Task::none()
            }

            Message::WindowDiscovered(id) => {
                self.window_id = id;
                if self.visible {
                    Task::none()
                } else {
                    self.minimize_task(true)
                }
            }
            Message::HideWindow => {
                self.visible = false;
                self.minimize_task(true)
            }

            Message::MinimizeDiscovered(minimized, id) => {
                self.window_id = id;
                match id {
                    Some(id) => iced::window::minimize(id, minimized),
                    None => Task::none(),
                }
            }

            Message::Exit => crate::windows::tray::force_exit(),
            Message::OpenConfigFolder => {
                if let Some(parent) = self.config_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                    let _ = std::process::Command::new("explorer.exe")
                        .arg(parent)
                        .spawn();
                }
                Task::none()
            }
            Message::TrayEvent(TrayEvent::Show) => {
                self.visible = true;
                self.minimize_task(false)
            }
            Message::TrayEvent(TrayEvent::Hide) => {
                self.visible = false;
                self.minimize_task(true)
            }
            Message::TrayEvent(TrayEvent::Exit) => Task::perform(async {}, |_| Message::Exit),
        }
    }

    fn minimize_task(&self, minimized: bool) -> Task<Message> {
        match self.window_id {
            Some(id) => iced::window::minimize(id, minimized),
            None => {
                iced::window::get_oldest().map(move |id| Message::MinimizeDiscovered(minimized, id))
            }
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let active_theme = self
            .pending_theme
            .unwrap_or_else(|| self.config.get().theme_mode());
        let sidebar = container(
            column![
                text("Pulse").size(30),
                text("native windows monitor").size(12),
                iced::widget::horizontal_rule(1),
                nav_column(self.page),
                iced::widget::vertical_space().height(Length::Fill),
                theme_buttons(active_theme),
                text(format!(
                    "CPU {}  RAM {}",
                    format_short(self.latest.cpu.usage_percent),
                    format_short(self.latest.memory.usage_percent)
                ))
                .size(12),
                text(&self.status).size(11),
                button(text("Hide to tray").size(12)).on_press(Message::HideWindow),
                button(text("Exit").size(12)).on_press(Message::Exit),
            ]
            .spacing(10)
            .padding(16),
        )
        .width(210)
        .height(Length::Fill)
        .style(|theme: &Theme| container::Style {
            background: Some(iced::Background::Color(
                theme.palette().background.scale_alpha(0.96),
            )),
            border: iced::Border {
                color: theme.palette().text.scale_alpha(0.12),
                width: 1.0,
                radius: 0.0.into(),
            },
            ..container::Style::default()
        });
        let content = match self.page {
            Page::Overview => ui::dashboard::view(self),
            Page::Cpu => ui::cpu::view(self),
            Page::Memory => ui::memory::view(self),
            Page::Gpu => ui::gpu::view(self),
            Page::Storage => ui::storage::view(self),
            Page::Network => ui::network::view(self),
            Page::Processes => ui::processes::view(self),
            Page::Settings => ui::settings::view(self),
        };
        let main = column![top_bar(self), container(content).height(Length::Fill)].spacing(0);
        row![sidebar, main]
            .spacing(0)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn visible_processes(&self) -> impl Iterator<Item = &ProcessInfo> {
        let mut list: Vec<&ProcessInfo> = self
            .latest
            .processes
            .iter()
            .filter(|p| {
                self.filter.is_empty()
                    || p.name
                        .to_ascii_lowercase()
                        .contains(&self.filter.to_ascii_lowercase())
            })
            .collect();
        match self.sort {
            ProcessSort::Cpu => list.sort_by(|a, b| b.cpu_percent.total_cmp(&a.cpu_percent)),
            ProcessSort::Memory => list.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes)),
            ProcessSort::Name => list.sort_by(|a, b| a.name.cmp(&b.name)),
            ProcessSort::Pid => list.sort_by(|a, b| a.pid.cmp(&b.pid)),
        }
        list.truncate(self.config.get().max_process_rows);
        list.into_iter()
    }
}

fn nav_column<'a>(active: Page) -> iced::widget::Column<'a, Message> {
    let mut out = iced::widget::column![].spacing(6);
    for page in Page::ALL {
        let item = button(text(page.label()).size(14))
            .width(Length::Fill)
            .on_press(Message::Navigate(page));
        out = out.push(if page == active {
            item.style(iced::widget::button::primary)
        } else {
            item
        });
    }
    out
}

fn theme_buttons<'a>(active: ThemeMode) -> iced::widget::Row<'a, Message> {
    let dark = button(text("Dark").size(12)).on_press(Message::ThemeChanged(ThemeMode::Dark));
    let light = button(text("Light").size(12)).on_press(Message::ThemeChanged(ThemeMode::Light));
    row![
        if active == ThemeMode::Dark {
            dark.style(iced::widget::button::primary)
        } else {
            dark
        },
        if active == ThemeMode::Light {
            light.style(iced::widget::button::primary)
        } else {
            light
        },
    ]
    .spacing(8)
}

fn top_bar(app: &PulseApp) -> Element<'_, Message> {
    container(
        row![
            text(app.page.label()).size(20),
            iced::widget::horizontal_space().width(Length::Fill),
            text(format!(
                "{} • {}",
                app.system.hostname, app.system.os_version
            ))
            .size(12),
            text(format!(
                "R {} • W {}",
                format_rate(app.latest.disk.read_bytes_per_sec),
                format_rate(app.latest.disk.write_bytes_per_sec)
            ))
            .size(12),
            text(format!(
                "↑ {} • ↓ {}",
                format_rate(app.latest.network.upload_bytes_per_sec),
                format_rate(app.latest.network.download_bytes_per_sec)
            ))
            .size(12),
            text(app.last_update.as_str()).size(12),
        ]
        .spacing(16)
        .padding(12),
    )
    .style(|theme: &Theme| container::Style {
        background: Some(iced::Background::Color(
            theme.palette().background.scale_alpha(0.92),
        )),
        border: iced::Border {
            color: theme.palette().text.scale_alpha(0.08),
            width: 1.0,
            radius: 0.0.into(),
        },
        ..container::Style::default()
    })
    .into()
}

fn apply_startup(enabled: bool) -> Result<()> {
    if enabled {
        crate::windows::startup::enable()
    } else {
        crate::windows::startup::disable()
    }
}

fn sort_name(sort: ProcessSort) -> String {
    match sort {
        ProcessSort::Cpu => "cpu",
        ProcessSort::Memory => "memory",
        ProcessSort::Name => "name",
        ProcessSort::Pid => "pid",
    }
    .to_string()
}

fn format_short(value: f32) -> String {
    format!("{value:.0}%")
}

fn rate_to_percent(bytes_per_sec: f64) -> f32 {
    (bytes_per_sec / (100.0 * 1024.0 * 1024.0) * 100.0).clamp(0.0, 100.0) as f32
}
