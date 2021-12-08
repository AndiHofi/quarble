use std::num::NonZeroU32;
use crate::ui::entry_edit::EntryEdit;
use crate::ui::util::{focus_next_ed, focus_previous, valid_end_time, valid_start_time};
use crate::ui::{style, Message, QElement};
use iced_winit::alignment::Horizontal;
use iced_winit::widget::{text_input, Row, Space, Text, TextInput};
use iced_winit::Length;
use crate::ui::time::Time;

pub struct WorkEntryEdit {
    id: usize,
    edit: bool,
    min_time: Time,
    max_time: Time,
    time_resolution: NonZeroU32,
    start_state: text_input::State,
    start_valid: bool,
    start: String,
    end_state: text_input::State,
    end_valid: bool,
    end: String,
    issue_state: text_input::State,
    issue: String,
    description_state: text_input::State,
    description: String,
}

impl WorkEntryEdit {
    pub fn new(id: usize) -> Box<Self> {
        Box::new(Self {
            id,
            edit: true,
            min_time: Time::hm(0, 0),
            max_time: Time::hm(24, 0),
            time_resolution: NonZeroU32::new(15).unwrap(),
            start_state: Default::default(),
            start_valid: false,
            start: "".to_string(),
            end_state: Default::default(),
            end_valid: false,
            end: "".to_string(),
            issue_state: Default::default(),
            issue: "".to_string(),
            description_state: Default::default(),
            description: "".to_string(),
        })
    }
}

impl EntryEdit for WorkEntryEdit {
    fn show<'a, 'b: 'a>(&'b mut self) -> crate::ui::QElement<'a> {
        let mut row = Row::new();
        let id: usize = self.id;
        row = row
            .push(
                Text::new("W")
                    .horizontal_alignment(Horizontal::Right)
                    .width(style::LABEL_WIDTH),
            )
            .push(Space::with_width(style::SPACE))
            .push(
                TextInput::new(&mut self.start_state, "", &self.start, move |i| {
                    valid_start_time(id, 0, i)
                })
                .width(style::TIME_WIDTH),
            )
            .push(Space::with_width(style::SPACE))
            .push(
                TextInput::new(&mut self.end_state, "", &self.end, move |i| {
                    valid_end_time(id, 0, i)
                })
                .width(style::TIME_WIDTH),
            )
            .push(Space::with_width(style::SPACE))
            .push(
                TextInput::new(
                    &mut self.description_state,
                    "",
                    &self.description,
                    move |input| Message::UpdateDescription { id, input },
                )
                .width(style::DESCRIPTION_WIDTH),
            )
            .push(Space::with_width(Length::Units(300)));

        QElement::new(row)
    }

    fn update_id(&mut self, id: usize) {
        self.id = id;
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::UpdateStart { input, .. } => {
                self.start = input;
                None
            }
            Message::UpdateEnd { input, .. } => {
                self.end = input;
                None
            }
            Message::UpdateDescription { input, .. } => {
                self.description = input;
                None
            }
            Message::Next => focus_next_ed(&mut [
                &mut self.start_state,
                &mut self.end_state,
                &mut self.description_state,
            ]),
            Message::Previous => focus_previous(&mut [
                &mut self.start_state,
                &mut self.end_state,
                &mut self.description_state,
            ]),
            Message::Up | Message::Down => {
                None
            }
            _ => None,
        }
    }

    fn has_focus(&self) -> bool {
        self.start_state.is_focused()
            || self.end_state.is_focused()
            || self.description_state.is_focused()
    }
}
