use crate::ui::entry_edit::EntryEdit;
use crate::ui::{style, util, Message};
use iced_winit::alignment::Horizontal;
use iced_winit::widget::{text_input, Row, Space, Text, TextInput};
use iced_winit::Length;

pub struct WorkStartEdit {
    id: usize,
    edit: bool,
    start_state: text_input::State,
    start: String,
    description_state: text_input::State,
    description: String,
}

impl WorkStartEdit {
    pub(crate) fn new(id: usize) -> Box<Self> {
        Box::new(Self {
            id,
            edit: true,
            start_state: Default::default(),
            start: "".to_string(),
            description_state: Default::default(),
            description: "".to_string(),
        })
    }
}

impl EntryEdit for WorkStartEdit {
    fn show<'a, 'b: 'a>(&'b mut self) -> crate::ui::QElement<'a> {
        let mut row = Row::new();
        let id: usize = self.id;
        row = row
            .push(
                Text::new("S")
                    .horizontal_alignment(Horizontal::Right)
                    .width(style::LABEL_WIDTH),
            )
            .push(Space::with_width(style::SPACE))
            .push(
                TextInput::new(&mut self.start_state, "", &self.start, move |input| {
                    util::valid_start_time(id, 0, input)
                })
                .width(style::TIME_WIDTH),
            )
            .push(Space::with_width(style::SPACE))
            .push(Space::with_width(style::TIME_WIDTH))
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
            .push(Space::with_width(Length::Fill));

        row.into()
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
            Message::UpdateDescription { input, .. } => {
                self.description = input;
                None
            }
            Message::Next => {
                util::focus_next_ed(&mut [&mut self.start_state, &mut self.description_state])
            }
            Message::Previous => {
                util::focus_previous(&mut [&mut self.start_state, &mut self.description_state])
            }
            _ => None,
        }
    }

    fn has_focus(&self) -> bool {
        self.start_state.is_focused() || self.description_state.is_focused()
    }
}
