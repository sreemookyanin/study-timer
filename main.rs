mod clock;
mod storage;
mod stat;
use iced::theme;

use clock::{format_time, Clock, CountdownTimer, Stopwatch};
use iced::widget::{button, column, container, pick_list, row, text};
use iced::{executor, Alignment, Application, Command, Element, Length, Settings, Subscription, Theme};
use std::time::Duration;

fn main() -> iced::Result {
    StudyTimer::run(Settings::default())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Screen {
    Timer,
    Stopwatch,
    Stat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Category {
    Exam,
    Study,
    Exercise,
    Relax,
}

impl Category {
    const ALL: [Category; 4] = [Category::Exam, Category::Study, Category::Exercise, Category::Relax];
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let name = match self {
            Category::Exam => "Exam",
            Category::Study => "Study",
            Category::Exercise => "Exercise",
            Category::Relax => "Relax",
        };
        write!(f, "{}", name)
    }
}

struct StudyTimer {
    screen: Screen,
    timer: CountdownTimer,
    stopwatch: Stopwatch,
    session_seconds: u32,
    show_dialog: bool,
    selected: Option<Category>,
    message: Option<String>,
    stats: Option<storage::Stats>,
    loading: bool,
    sort_longest: bool,

}

#[derive(Debug, Clone)]
enum Message {
    Switch(Screen),
    Add(u32),
    Start,
    Stop,
    Reset,
    Tick,
    CategoryPicked(Category),
    ConfirmSave,
    CancelSave,
    Saved(Result<(), String>),
    ToggleSort,
    StatsLoaded(Result<storage::Stats, String>),

}

impl StudyTimer {
    fn active(&self) -> Option<&dyn Clock> {
        match self.screen {
            Screen::Timer => Some(&self.timer),
            Screen::Stopwatch => Some(&self.stopwatch),
            Screen::Stat => None,
        }
    }
    fn load_stats(&mut self) -> Command<Message> {
        self.loading = true;
        let url = std::env::var("DATABASE_URL").unwrap_or_default();
        Command::perform(
            async move {
                tokio::task::spawn_blocking(move || storage::load_stats(&url))
                    .await
                    .unwrap_or_else(|e| Err(e.to_string()))
            },
            Message::StatsLoaded,
        )
}


    fn active_mut(&mut self) -> Option<&mut dyn Clock> {
        match self.screen {
            Screen::Timer => Some(&mut self.timer),
            Screen::Stopwatch => Some(&mut self.stopwatch),
            Screen::Stat => None,
        }
    }

    fn stat_view(&self) -> Element<Message> {
        if self.loading {
            return text("Loading…").into();
        }
        let Some(stats) = &self.stats else {
            return text("No data yet").into();
        };
        if stats.recent.is_empty() {
            return text("No sessions saved yet").into();
        }

        // Left: table of sessions
        let mut rows = stats.recent.clone();
        if self.sort_longest {
            stat::sort_longest(&mut rows);
        }
        let mut table = column![row![
            text("Session").width(Length::FillPortion(2)),
            text("Time").width(Length::FillPortion(1)),
        ]]
        .spacing(8);
        for (i, r) in rows.iter().enumerate() {
            table = table.push(row![
                text(format!("{}: {}", i + 1, r.category)).width(Length::FillPortion(2)),
                text(format_time(r.seconds)).width(Length::FillPortion(1)),
            ]);
        }
        let sort_label = if self.sort_longest { "Sort: Longest" } else { "Sort: Newest" };
        let left = column![button(sort_label).on_press(Message::ToggleSort), table]
            .spacing(15)
            .width(Length::FillPortion(1));

        // Right: bar chart, total time per mode
        let max = stats.totals.iter().map(|(_, s)| *s).max().unwrap_or(0);
        let mut bars = row![].spacing(20).align_items(Alignment::End);
        for (cat, secs) in &stats.totals {
            bars = bars.push(
                column![
                    text(format_time(*secs)).size(14),
                    container(text(""))
                        .width(Length::Fixed(40.0))
                        .height(Length::Fixed(stat::bar_height(*secs, max, 200.0)))
                        .style(theme::Container::Box),
                    text(cat.clone()),
                ]
                .spacing(5)
                .align_items(Alignment::Center),
            );
        }
        let right = column![text("Total time per mode"), bars]
            .spacing(15)
            .width(Length::FillPortion(1));

        row![left, right].spacing(40).into()
    }

}

impl Application for StudyTimer {
    type Executor = executor::Default;
    type Message = Message;
    type Theme = Theme;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        (
            StudyTimer {
                screen: Screen::Timer,
                timer: CountdownTimer::default(),
                stopwatch: Stopwatch::default(),
                session_seconds: 0,
                show_dialog: false,
                selected: None,
                message: None,
                stats: None,
                loading: false,
                sort_longest: false,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Study Timer")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Switch(s) => {
                self.screen = s;
                self.message = None;
                if s == Screen::Stat {
                    return self.load_stats();
                }
            }
            Message::ToggleSort => self.sort_longest = !self.sort_longest,
            Message::StatsLoaded(Ok(s)) => {
                self.loading = false;
                self.stats = Some(s);
            }
            Message::StatsLoaded(Err(e)) => {
                self.loading = false;
                self.message = Some(format!("Could not load stats: {e}"));
            }

            Message::Add(secs) => {
                self.timer.add(secs);
                self.message = None;
            }
            Message::Start => {
                if self.screen == Screen::Timer && self.timer.seconds() == 0 {
                    self.message = Some("Add some time first".into());
                } else {
                    if let Some(c) = self.active_mut() {
                        c.start();
                    }
                    self.message = None;
                }
            }
            Message::Stop => {
                if let Some(c) = self.active_mut() {
                    c.stop();
                }
                // Stopwatch: stopping opens the save dialog
                if self.screen == Screen::Stopwatch && self.stopwatch.session_seconds() > 0 {
                    self.session_seconds = self.stopwatch.session_seconds();
                    self.show_dialog = true;
                }
            }
            Message::Reset => {
                if let Some(c) = self.active_mut() {
                    c.reset();
                }
                self.message = None;
            }
            Message::Tick => match self.screen {
                Screen::Timer => {
                    self.timer.tick();
                    if self.timer.is_finished() {
                        self.session_seconds = self.timer.session_seconds();
                        self.show_dialog = true;
                    }
                }
                Screen::Stopwatch => self.stopwatch.tick(),
                Screen::Stat => {}
            },
            Message::CategoryPicked(c) => self.selected = Some(c),
            Message::CancelSave => {
                self.show_dialog = false;
                self.selected = None;
            }
            Message::ConfirmSave => {
                if let Some(cat) = self.selected {
                    let secs = self.session_seconds;
                    let url = std::env::var("DATABASE_URL").unwrap_or_default();
                    self.show_dialog = false;
                    self.selected = None;
                    if let Some(c) = self.active_mut() {
                        c.reset();
                    }
                    return Command::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                storage::save_session(&url, &cat.to_string(), secs)
                            })
                            .await
                            .unwrap_or_else(|e| Err(e.to_string()))
                        },
                        Message::Saved,
                    );
                }
            }
            Message::Saved(Ok(())) => self.message = Some("Session saved!".into()),
            Message::Saved(Err(e)) => self.message = Some(format!("Could not save: {e}")),
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        if self.show_dialog {
            return container(
                column![
                    text("Session end!").size(32),
                    text(format_time(self.session_seconds)),
                    row![
                        text("Choose mode"),
                        pick_list(&Category::ALL[..], self.selected, Message::CategoryPicked)
                            .placeholder("Select..."),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                    row![
                        button("Cancel").on_press(Message::CancelSave),
                        button("Confirm").on_press_maybe(self.selected.map(|_| Message::ConfirmSave)),
                    ]
                    .spacing(10),
                ]
                .spacing(20)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();
        }

        // Tabs are disabled while a clock is running
        let busy = self.timer.is_running() || self.stopwatch.is_running();
        let tab = |label: &str, s: Screen| {
            button(text(label.to_string())).on_press_maybe(if busy { None } else { Some(Message::Switch(s)) })
        };
        let tabs = row![
            tab("Timer", Screen::Timer),
            tab("Stopwatch", Screen::Stopwatch),
            tab("Stat", Screen::Stat)
        ]
        .spacing(10);

        let body: Element<Message> = match self.screen {
            Screen::Timer | Screen::Stopwatch => {
                let secs = self.active().map(|c| c.seconds()).unwrap_or(0);
                let mut col = column![text(format_time(secs)).size(64)]
                    .spacing(20)
                    .align_items(Alignment::Center);

                if self.screen == Screen::Timer {
                    let add = |label: &str, s: u32| {
                        button(text(label.to_string()))
                            .on_press_maybe(if self.timer.is_running() { None } else { Some(Message::Add(s)) })
                    };
                    col = col.push(row![add("+00:30", 30), add("+01:00", 60), add("+05:00", 300)].spacing(10));
                }

                col.push(
                    row![
                        button("Start").on_press(Message::Start),
                        button("Stop").on_press(Message::Stop),
                        button("Reset").on_press(Message::Reset),
                    ]
                    .spacing(10),
                )
                .into()
            }
            Screen::Stat => self.stat_view(),
        };

        let mut content = column![tabs, body].spacing(30).align_items(Alignment::Center);
        if let Some(m) = &self.message {
            content = content.push(text(m.clone()));
        }

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        if self.active().map_or(false, |c| c.is_running()) {
            iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }
}
