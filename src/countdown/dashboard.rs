use iced::Color;
use iced::border::Radius;
use iced::widget::button::Status::Hovered;
use iced::widget::button::Style;
use iced::widget::{button, container, stack, text};

use iced::{Element, Length, Subscription, Vector, time::*, *};

use crate::countdown::custom_widgets::circular_progress_bar::CircularProgressBar;
use crate::countdown::dashboard::DashboardMessage::{
    CountdownPartSelected, PauseCountdown, ResetCountdown, StartCountdown,
};

pub const BACKGROUND_COLOR: Color = Color::WHITE; // #0F172A

pub const TEXT_COLOR: Color = Color::from_rgb8(35, 48, 70); // #F8FAFC

pub const CIRCULAR_COLOR: Color = Color::from_rgb8(35, 48, 70); // #233046

pub const TRACK_COLOR: Color = Color::from_rgb8(248, 250, 252); // #F8FAFC

#[derive(Debug)]
pub struct Dashboard {
    // Time related variables
    hours: u64,
    minutes: u64,
    seconds: u64,

    // Focus related variables
    focus_duration: Duration,
    remaining_focus: Duration,

    // The currently selected part of the countdown timer (hours, minutes, seconds)
    selected_timer_part: Option<CountdownPart>,
    buffer: String,

    // Whether the timer is currently running, paused or in editing mode
    countdown_state: CountdownState,

    // Resize handling
    window_ratio: f32,
    window_size: Vector,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountdownPart {
    Hours,
    Minutes,
    Seconds,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountdownState {
    Running,
    Paused,
    Editing,
}

#[derive(Debug, Clone)]
pub enum DashboardMessage {
    Tick(Instant),
    ContentResize(Vector),
    CountdownTick,
    KeyboardInput(Event),
    CountdownPartSelected(CountdownPart),
    StartCountdown,
    PauseCountdown,
    ResetCountdown,
    CountdownFinished,
}

pub enum Action {
    None,
    // This component needs to run a task
    Run(iced::Task<DashboardMessage>),
}

impl Dashboard {
    pub fn new() -> Self {
        let initial_time = Duration::from_mins(0);

        Self {
            hours: 0,
            minutes: 0,
            seconds: 0,
            focus_duration: initial_time,
            remaining_focus: initial_time,
            selected_timer_part: None,
            buffer: String::new(),
            countdown_state: CountdownState::Editing,
            window_ratio: 1.0,
            window_size: Vector::new(0.0, 0.0),
        }
    }

    pub fn subscriptions() -> Subscription<DashboardMessage> {
        let subscriptions = [
            iced::time::every(Duration::from_secs_f32(1.0 / 60.0))
                .map(|_| DashboardMessage::Tick(Instant::now())),
            iced::window::resize_events()
                .map(|(_, size)| DashboardMessage::ContentResize(size.into())),
            iced::time::every(Duration::from_secs_f32(1.0))
                .map(|_| DashboardMessage::CountdownTick),
            iced::event::listen().map(DashboardMessage::KeyboardInput),
        ];

        Subscription::batch(subscriptions)
    }

    pub fn update(&mut self, msg: DashboardMessage) -> Action {
        use DashboardMessage::*;

        match msg {
            Tick(_now) => Action::None,
            ContentResize(size) => {
                self.window_ratio = size.x / size.y;
                self.window_size = size;

                Action::None
            }
            CountdownTick => {
                if self.countdown_state == CountdownState::Running {
                    if self.remaining_focus > Duration::ZERO {
                        self.remaining_focus =
                            self.remaining_focus.saturating_sub(Duration::from_secs(1));
                    } else {
                        return Action::Run(iced::Task::perform(
                            async { DashboardMessage::CountdownFinished },
                            |msg| msg,
                        ));
                    }
                }

                Action::None
            }
            KeyboardInput(event) => {
                use iced::keyboard::Key;
                use iced::keyboard::key::Named;

                if self.countdown_state == CountdownState::Editing
                    && let Event::Keyboard(k_event) = event
                    && let keyboard::Event::KeyPressed { key, .. } = k_event
                {
                    let Some(part) = self.selected_timer_part else {
                        return Action::None;
                    };

                    match key {
                        Key::Character(c) => {
                            for ch in c.chars() {
                                if !ch.is_ascii_digit() {
                                    return Action::None;
                                }

                                self.buffer.push(ch);

                                if self.buffer.len() > 2 {
                                    self.buffer.remove(0);
                                }

                                let value: u64 = self.buffer.parse().unwrap_or(0);

                                match part {
                                    CountdownPart::Hours => {
                                        self.hours = value.min(99);
                                    }
                                    CountdownPart::Minutes => {
                                        self.minutes = value.min(59);
                                    }
                                    CountdownPart::Seconds => {
                                        self.seconds = value.min(59);
                                    }
                                    CountdownPart::None => {}
                                }
                            }
                        }
                        Key::Named(Named::ArrowUp) => match part {
                            CountdownPart::Hours => self.hours = (self.hours + 1).min(99),
                            CountdownPart::Minutes => self.minutes = (self.minutes + 1).min(59),
                            CountdownPart::Seconds => self.seconds = (self.seconds + 1).min(59),
                            CountdownPart::None => {}
                        },
                        Key::Named(Named::ArrowDown) => match part {
                            CountdownPart::Hours => self.hours -= 1,
                            CountdownPart::Minutes => self.minutes -= 1,
                            CountdownPart::Seconds => self.seconds -= 1,
                            CountdownPart::None => {}
                        },
                        _ => {}
                    }
                }
                Action::None
            }
            CountdownPartSelected(timer_part) => {
                self.selected_timer_part = Some(timer_part);
                self.buffer.clear();

                Action::None
            }
            StartCountdown => {
                let input_duration =
                    Duration::from_secs(self.hours * 3600 + self.minutes * 60 + self.seconds);

                if input_duration == Duration::ZERO && self.remaining_focus == Duration::ZERO {
                    return Action::None;
                }

                self.countdown_state = CountdownState::Running;

                if self.focus_duration == Duration::ZERO {
                    self.focus_duration = input_duration;
                }

                if self.remaining_focus == Duration::ZERO {
                    self.remaining_focus = self.focus_duration;
                }

                Action::None
            }
            PauseCountdown => {
                self.countdown_state = CountdownState::Paused;

                Action::None
            }
            ResetCountdown => {
                self.hours = 0;
                self.minutes = 0;
                self.seconds = 0;

                self.countdown_state = CountdownState::Editing;

                self.focus_duration = Duration::ZERO;
                self.remaining_focus = Duration::ZERO;

                Action::None
            }
            CountdownFinished => {
                self.countdown_state = CountdownState::Editing;

                self.hours = 0;
                self.minutes = 0;
                self.seconds = 0;

                self.focus_duration = Duration::ZERO;
                self.remaining_focus = Duration::ZERO;

                Action::None
            }
        }
    }

    pub fn view(&self) -> Element<'_, DashboardMessage> {
        use iced::widget::{column, container, mouse_area, row, space, text};

        let font_size = 64.0;

        let color = |selected_timer_part: CountdownPart| {
            if let Some(selected) = self.selected_timer_part {
                if selected == selected_timer_part {
                    iced::Color::from_rgb8(96, 165, 250)
                } else {
                    TEXT_COLOR
                }
            } else {
                TEXT_COLOR
            }
        };

        let hours = if self.countdown_state == CountdownState::Editing {
            self.hours
        } else {
            self.remaining_focus.as_secs() / 3600
        };
        let minutes = if self.countdown_state == CountdownState::Editing {
            self.minutes
        } else {
            (self.remaining_focus.as_secs() % 3600) / 60
        };
        let seconds = if self.countdown_state == CountdownState::Editing {
            self.seconds
        } else {
            self.remaining_focus.as_secs() % 60
        };

        let hours_raw_text = text(if hours < 10 {
            "0".to_string() + &hours.to_string()
        } else {
            hours.to_string()
        })
        .size(font_size * self.window_ratio)
        .color(color(CountdownPart::Hours));

        let hours_text = if self.countdown_state == CountdownState::Editing {
            container(
                mouse_area(hours_raw_text)
                    .on_press(CountdownPartSelected(CountdownPart::Hours))
                    .on_exit(CountdownPartSelected(CountdownPart::None))
                    .interaction(iced::mouse::Interaction::Pointer),
            )
        } else {
            container(hours_raw_text)
        };

        let minutes_raw_text = text(if minutes < 10 {
            "0".to_string() + &minutes.to_string()
        } else {
            minutes.to_string()
        })
        .size(font_size * self.window_ratio)
        .color(color(CountdownPart::Minutes));

        let minutes_text = if self.countdown_state == CountdownState::Editing {
            container(
                mouse_area(minutes_raw_text)
                    .on_press(CountdownPartSelected(CountdownPart::Minutes))
                    .on_exit(CountdownPartSelected(CountdownPart::None))
                    .interaction(iced::mouse::Interaction::Pointer),
            )
        } else {
            container(minutes_raw_text)
        };

        let second_raw_text = text(if seconds < 10 {
            "0".to_string() + &seconds.to_string()
        } else {
            seconds.to_string()
        })
        .size(font_size * self.window_ratio)
        .color(color(CountdownPart::Seconds));

        let seconds_text = if self.countdown_state == CountdownState::Editing {
            container(
                mouse_area(second_raw_text)
                    .on_press(CountdownPartSelected(CountdownPart::Seconds))
                    .on_exit(CountdownPartSelected(CountdownPart::None))
                    .interaction(iced::mouse::Interaction::Pointer),
            )
        } else {
            container(second_raw_text)
        };

        let circular_size = 310.0 * self.window_ratio;

        let circular_progress_bar = CircularProgressBar {
            circular_size,
            bar_height: 10.0 * self.window_ratio,
            circular_color: CIRCULAR_COLOR,
            track_color: TRACK_COLOR,
            progress: 1.0 - self.remaining_focus.as_secs_f32() / self.focus_duration.as_secs_f32(),
        };

        let show_hours = self.focus_duration.as_secs() >= 3600;

        let mut time_text_row = row![].spacing(0).padding(Padding::new(0.0));

        if self.countdown_state != CountdownState::Editing {
            if show_hours {
                time_text_row = time_text_row.push(hours_text).push(
                    text(":")
                        .size(font_size * self.window_ratio)
                        .color(TEXT_COLOR),
                );
            }
        } else {
            time_text_row = time_text_row.push(hours_text).push(
                text(":")
                    .size(font_size * self.window_ratio)
                    .color(TEXT_COLOR),
            );
        }

        time_text_row = time_text_row
            .push(minutes_text)
            .push(
                text(":")
                    .size(font_size * self.window_ratio)
                    .color(TEXT_COLOR),
            )
            .push(seconds_text);

        let time_container = container(time_text_row)
            .center_x(Length::Fixed(circular_size))
            .center_y(Length::Fixed(circular_size));

        let button_row = row![
            start_button(self.window_ratio, self.countdown_state),
            pause_reset_button(self.window_ratio, self.countdown_state),
        ]
        .spacing(12.0 * self.window_ratio)
        .padding([10.0 * self.window_ratio, 22.0 * self.window_ratio]);

        let button_container = container(column![
            space().height(Length::FillPortion(3)),
            button_row,
            space().height(Length::FillPortion(1)),
        ])
        .center_x(Length::Fixed(circular_size))
        .center_y(Length::Fixed(circular_size));

        let countdown = stack![circular_progress_bar, time_container, button_container];

        container(countdown)
            .style(move |_| iced::widget::container::Style {
                background: Some(BACKGROUND_COLOR.into()),
                ..Default::default()
            })
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }
}

#[inline]
fn start_button<'a>(
    window_ratio: f32,
    state: CountdownState,
) -> iced::widget::Button<'a, DashboardMessage> {
    button(
        container(text("Start").size(16.0 * window_ratio))
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill),
    )
    .width(65.0 * window_ratio)
    .height(36.0 * window_ratio)
    .padding(0)
    .style(move |_, status| {
        let start_bg = Color::from_rgb8(248, 251, 255);
        let start_border = Color::from_rgb8(191, 219, 254);
        let start_text = Color::from_rgb8(59, 130, 246);

        let start_bg_hover = Color::from_rgb8(243, 248, 255);
        let start_border_hover = Color::from_rgb8(147, 197, 253);

        let background = match status {
            Hovered => Some(start_bg_hover.into()),
            _ => Some(start_bg.into()),
        };

        let border = match status {
            Hovered => start_border_hover,
            _ => start_border,
        };

        Style {
            background,
            text_color: start_text,
            border: Border {
                radius: Radius::new(18.0 * window_ratio),
                color: border,
                ..Default::default()
            },

            ..Default::default()
        }
    })
    .on_press_maybe(
        if state == CountdownState::Editing || state == CountdownState::Paused {
            Some(StartCountdown)
        } else {
            None
        },
    )
}

#[inline]
fn pause_reset_button<'a>(
    window_ratio: f32,
    state: CountdownState,
) -> iced::widget::Button<'a, DashboardMessage> {
    let pause_reset_text = match state {
        CountdownState::Paused => "Reset",
        _ => "Pause",
    };

    button(
        container(text(pause_reset_text).size(16.0 * window_ratio))
            .center_x(iced::Length::Fill)
            .center_y(iced::Length::Fill),
    )
    .width(65.0 * window_ratio)
    .height(36.0 * window_ratio)
    .padding(0)
    .style(move |_, status| {
        let pause_bg = Color::from_rgb8(255, 252, 245);
        let pause_border = Color::from_rgb8(241, 229, 199);
        let pause_text = Color::from_rgb8(180, 145, 82);

        let pause_bg_hover = Color::from_rgb8(255, 249, 238);
        let pause_border_hover = Color::from_rgb8(232, 216, 175);

        let background = match status {
            Hovered => Some(pause_bg_hover.into()),
            _ => Some(pause_bg.into()),
        };

        let border = match status {
            Hovered => pause_border_hover,
            _ => pause_border,
        };

        Style {
            background,
            text_color: pause_text,
            border: Border {
                radius: Radius::new(18.0 * window_ratio),
                color: border,
                ..Default::default()
            },

            ..Default::default()
        }
    })
    .on_press(if state == CountdownState::Running {
        PauseCountdown
    } else {
        ResetCountdown
    })
}
