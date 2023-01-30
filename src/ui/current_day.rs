use iced_core::alignment::Horizontal;
use iced_core::Length;
use iced_native::widget::{button, text_input, tree, TextInput};
use iced_native::Widget;
use iced_winit::theme;
use iced_winit::widget::{scrollable, Column, Container, Row, Scrollable, Space, Text};

use crate::conf::SettingsRef;
use crate::data::{Action, ActiveDay, Day};
use crate::parsing::time::Time;
use crate::ui::message::{DeleteAction, EditAction};
use crate::ui::my_text_input::MyTextInput;
use crate::ui::stay_active::StayActive;
use crate::ui::util::h_space;
use crate::ui::{style, text};
use crate::ui::{MainView, Message, QElement};

#[derive(Clone, Debug)]
pub enum CurrentDayMessage {
    DayTextChanged(String),
    StartDayChange,
    CommitDayChange,
    RequestEdit(usize),
    RequestDelete(usize),
}

#[derive(Debug)]
pub struct CurrentDayUI {
    data: ActiveDay,
    scroll_state: scrollable::State,
    day_select_button: button::State,
    editing_current_day: bool,
    day_value: MyTextInput,
    settings: SettingsRef,
    entries: Vec<Entry>,
    selected_entry: Option<usize>,
}

#[derive(Clone, Debug)]
struct Entry {
    id: usize,
    edit_button: button::State,
    delete_button: button::State,
    action: Action,
}

impl CurrentDayUI {
    pub fn for_active_day(settings: SettingsRef, active_day: Option<&ActiveDay>) -> Box<Self> {
        let entries = if let Some(e) = active_day {
            e.actions()
                .iter()
                .cloned()
                .enumerate()
                .map(|(id, action)| Entry {
                    id,
                    edit_button: button::State::new(),
                    delete_button: button::State::new(),
                    action,
                })
                .collect()
        } else {
            Vec::new()
        };
        Box::new(Self {
            data: active_day.cloned().unwrap_or_default(),
            scroll_state: Default::default(),
            day_select_button: button::State::new(),
            editing_current_day: false,
            day_value: MyTextInput::new("", |_| true),
            settings,
            entries,
            selected_entry: None,
        })
    }
}

impl MainView for CurrentDayUI {
    fn view(&self) -> QElement {
        let day = self.data.get_day().to_string();

        let active_issue: Row<_, _> = if let Some(active_issue) = self.data.active_issue() {
            Row::with_children(vec![
                text(&active_issue.ident),
                h_space(style::DSPACE),
                text(active_issue.default_action.as_deref().unwrap_or_default()),
                h_space(style::DSPACE),
                text(active_issue.description.as_deref().unwrap_or_default()),
            ])
        } else {
            Row::with_children(vec![text("No active issue")])
        };

        let entries: Vec<QElement> = self
            .entries
            .iter()
            .enumerate()
            .map(|(index, e)| edit_action_row(e, index, self.selected_entry))
            .collect();

        let mut entries_scroll =
            Scrollable::new(Column::with_children(entries).width(Length::Fill));

        let date_width = Length::Units(100);
        let mut day_row = Vec::new();
        let (on_press, message) = if self.editing_current_day {
            (Message::Cd(CurrentDayMessage::CommitDayChange), "Commit")
        } else {
            (
                Message::Cd(CurrentDayMessage::StartDayChange),
                "Change day (d)",
            )
        };
        if self.editing_current_day {
            let on_submit = Message::Cd(CurrentDayMessage::CommitDayChange);
            let input = self
                .day_value
                .show_text_input(date_width)
                .on_submit(on_submit);
            day_row.push(input.into())
        } else {
            day_row.push(
                Text::new(day)
                    .width(date_width)
                    .horizontal_alignment(Horizontal::Left)
                    .into(),
            )
        };
        day_row.push(h_space(style::DSPACE));
        day_row.push(style::inline_button(message).on_press(on_press).into());

        Column::with_children(vec![
            Row::with_children(day_row).into(),
            Space::with_height(style::SPACE).into(),
            active_issue.into(),
            Space::with_height(style::SPACE).into(),
            Container::new(entries_scroll)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(style::container_style(style::ContentStyle))
                .padding([5, 1])
                .into(),
        ])
        .into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Input(id, input) if id == self.day_value.id => {
                self.day_value.text = input;
                None
            }
            Message::Cd(CurrentDayMessage::StartDayChange) => {
                self.editing_current_day = true;
                Some(Message::ForceFocus(self.day_value.id.clone()))
            }
            Message::Cd(CurrentDayMessage::CommitDayChange) => {
                if self.day_value.text.is_empty() {
                    self.editing_current_day = false;
                    None
                } else {
                    let parsed = Day::parse_day_relative(
                        &self.settings.load().timeline,
                        &self.day_value.text,
                    );
                    parsed.get().map(Message::ChangeDay)
                }
            }
            Message::Cd(CurrentDayMessage::RequestEdit(id)) => self
                .entries
                .get(id)
                .map(|e| Message::EditAction(EditAction(Box::new(e.action.clone())))),
            Message::Cd(CurrentDayMessage::RequestDelete(id)) => self.entries.get(id).map(|e| {
                Message::DeleteAction(DeleteAction(StayActive::Yes, Box::new(e.action.clone())))
            }),
            Message::Up => {
                self.selected_entry = match self.selected_entry {
                    None | Some(0) => Some(self.entries.len() - 1),
                    Some(index) => Some(index - 1),
                };
                None
            }
            Message::Down => {
                self.selected_entry = match self.selected_entry {
                    None => Some(0),
                    Some(index) if index >= self.entries.len() - 1 => Some(0),
                    Some(index) => Some(index + 1),
                };
                None
            }
            Message::SubmitCurrent(_) => self
                .selected_entry
                .map(|e| Message::Cd(CurrentDayMessage::RequestEdit(e))),
            Message::Del => self
                .selected_entry
                .map(|e| Message::Cd(CurrentDayMessage::RequestDelete(e))),
            _ => None,
        }
    }
}

fn edit_action_row(entry: &Entry, index: usize, selected_index: Option<usize>) -> QElement {
    let delete_button =
        style::inline_button("D").on_press(Message::Cd(CurrentDayMessage::RequestDelete(entry.id)));
    let edit_button =
        style::inline_button("E").on_press(Message::Cd(CurrentDayMessage::RequestEdit(entry.id)));
    let background = style::ContentRow {
        state: if Some(index) == selected_index {
            style::RowState::Selected
        } else if index % 2 == 1 {
            style::RowState::Odd
        } else {
            style::RowState::Even
        },
    };

    Container::new(Row::with_children(vec![
        delete_button.into(),
        h_space(Length::Units(3)),
        edit_button.into(),
        h_space(style::DSPACE),
        action_row(&entry.action),
    ]))
    .style(theme::Container::Custom(Box::new(background)))
    .width(Length::Fill)
    .padding([2, 5])
    .into()
}

pub fn action_row<'a>(action: &'a Action) -> QElement<'a> {
    let w = Length::Units(50);
    let s = Length::Units(35);
    let time = |t: Time| {
        Text::new(t.to_string())
            .width(w)
            .horizontal_alignment(Horizontal::Right)
    };
    let dash = |sep: &'a str| {
        Text::new(sep)
            .width(s)
            .horizontal_alignment(Horizontal::Center)
    };

    let mut row = Vec::<QElement>::new();
    match (action.start(), action.end()) {
        (Some(start), Some(end)) => {
            row.push(time(start).into());
            row.push(dash("-").into());
            row.push(time(end).into());
        }
        (Some(start), None) => {
            row.push(time(start).into());
            row.push(dash("-").into());
            row.push(h_space(w));
        }
        (None, Some(end)) => {
            row.push(h_space(w));
            row.push(dash("-").into());
            row.push(time(end).into());
        }
        (None, None) => row.push(
            Text::new("all day")
                .horizontal_alignment(Horizontal::Center)
                .width(Length::Units(140))
                .into(),
        ),
    }

    row.push(dash(" | ").into());

    if let Some(id) = action.issue_id() {
        row.push(
            Text::new(id)
                .width(Length::Units(120))
                .horizontal_alignment(Horizontal::Left)
                .into(),
        );
        row.push(dash(":").into());
    } else {
        row.push(h_space(Length::Units(120)));
    }

    row.push(Text::new(action.as_no_time().to_string()).into());

    Row::with_children(row).into()
}
