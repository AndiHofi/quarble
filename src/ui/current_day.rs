use crate::conf::Settings;
use crate::data::{Action, ActiveDay, Day};
use crate::db::DB;
use crate::parsing::time::Time;
use crate::ui::util::h_space;
use crate::ui::{style, text, SettingsRef};
use crate::ui::{MainView, Message, QElement};
use iced_core::alignment::Horizontal;
use iced_core::Length;
use iced_native::widget::{button, text_input, Button};
use iced_wgpu::TextInput;
use iced_winit::widget::{scrollable, Column, Container, Row, Scrollable, Space, Text};

#[derive(Clone, Debug)]
pub enum CurrentDayMessage {
    DayTextChanged(String),
    StartDayChange,
    CommitDayChange,
}

#[derive(Clone, Debug)]
pub struct CurrentDayUI {
    data: ActiveDay,
    scroll_state: scrollable::State,
    day_select_button: button::State,
    edit_state: Option<text_input::State>,
    day_value: String,
    settings: SettingsRef,
}

impl CurrentDayUI {
    pub fn for_active_day(settings: SettingsRef, active_day: Option<&ActiveDay>) -> Box<Self> {
        Box::new(Self {
            data: active_day.cloned().unwrap_or_default(),
            scroll_state: Default::default(),
            day_select_button: button::State::new(),
            edit_state: None,
            day_value: String::new(),
            settings,
        })
    }

    fn action_row<'a>(&'a self, action: &Action) -> QElement<'static> {
        let w = Length::Units(40);
        let s = Length::Units(40);
        let time = |t: Time| {
            Text::new(t.to_string())
                .width(w)
                .horizontal_alignment(Horizontal::Right)
                .into()
        };
        let dash = |sep: &str| {
            Text::new(sep)
                .width(s)
                .horizontal_alignment(Horizontal::Center)
                .into()
        };

        let mut row = Vec::new();
        match (action.start(), action.end()) {
            (Some(start), Some(end)) => {
                row.push(time(start));
                row.push(dash("-"));
                row.push(time(end));
            }
            (Some(start), None) => {
                row.push(time(start));
                row.push(dash("-"));
                row.push(h_space(w));
            }
            (None, Some(end)) => {
                row.push(h_space(w));
                row.push(dash("-"));
                row.push(time(end));
            }
            (None, None) => row.push(
                Text::new("all day")
                    .horizontal_alignment(Horizontal::Center)
                    .width(Length::Units(140))
                    .into(),
            ),
        }

        row.push(dash(" | "));

        if let Some(id) = action.issue_id() {
            row.push(
                Text::new(id)
                    .width(Length::Units(80))
                    .horizontal_alignment(Horizontal::Left)
                    .into(),
            );
            row.push(dash(":"));
        } else {
            row.push(h_space(Length::Units(120)));
        }

        row.push(
            Text::new(action.as_no_time().to_string())
                .width(Length::Fill)
                .into(),
        );

        Row::with_children(row).into()
    }
}

impl MainView for CurrentDayUI {
    fn view(&mut self, _settings: &Settings) -> QElement<'_> {
        let day = self.data.get_day().to_string();
        let active_issue = self
            .data
            .active_issue()
            .map(|i| i.to_string())
            .unwrap_or_else(|| "No active issue".to_string());

        let entries: Vec<QElement<'static>> = self
            .data
            .actions()
            .iter()
            .map(|e| self.action_row(e))
            .collect();
        let mut entries_scroll = Scrollable::new(&mut self.scroll_state);
        for e in entries {
            entries_scroll = entries_scroll.push(e);
        }
        let content_style: Box<dyn iced_winit::widget::container::StyleSheet> =
            Box::new(style::ContentStyle);

        let date_width = Length::Units(100);
        let mut day_row = Vec::new();
        let (on_press, message) = if self.edit_state.is_some() {
            (Message::Cd(CurrentDayMessage::CommitDayChange), "Commit")
        } else {
            (Message::Cd(CurrentDayMessage::StartDayChange), "Change day")
        };
        if let Some(edit_state) = &mut self.edit_state {
            let on_submit = Message::Cd(CurrentDayMessage::CommitDayChange);
            day_row.push(
                TextInput::new(edit_state, &day, &mut self.day_value, |v| {
                    Message::Cd(CurrentDayMessage::DayTextChanged(v))
                })
                .on_submit(on_submit)
                .width(date_width)
                .into(),
            )
        } else {
            day_row.push(
                Text::new(day)
                    .width(date_width)
                    .horizontal_alignment(Horizontal::Left)
                    .into(),
            )
        };
        day_row.push(h_space(style::DSPACE));
        day_row.push(
            Button::new(&mut self.day_select_button, text(message))
                .on_press(on_press)
                .into(),
        );

        Column::with_children(vec![
            Row::with_children(day_row).into(),
            Space::with_height(style::SPACE).into(),
            Text::new(active_issue).into(),
            Space::with_height(style::SPACE).into(),
            Container::new(entries_scroll)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(content_style)
                .padding(style::SPACE_PX)
                .into(),
        ])
        .into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Cd(CurrentDayMessage::DayTextChanged(input)) => {
                self.day_value = input;
                None
            }
            Message::Cd(CurrentDayMessage::StartDayChange) => {
                if self.edit_state.is_none() {
                    self.edit_state = Some(text_input::State::focused());
                }
                None
            }
            Message::Cd(CurrentDayMessage::CommitDayChange) => {
                if self.day_value.is_empty() {
                    self.edit_state = None;
                    None
                } else {
                    let parsed =
                        Day::parse_day_relative(&self.settings.load().timeline, &self.day_value);
                    parsed.get().map(Message::ChangeDay)
                }
            }
            _ => None,
        }
    }
}
