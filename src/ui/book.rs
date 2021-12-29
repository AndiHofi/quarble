use crate::conf::Settings;
use crate::ui::entry_edit::EntryEdit;
use crate::ui::work_entry_edit::WorkEntryEdit;
use crate::ui::work_start_edit::WorkStartEdit;
use crate::ui::Message::{UpdateDescription, UpdateEnd, UpdateStart};
use crate::ui::{MainView, Message, QElement};
use iced_winit::widget::{scrollable, Column, Row, Rule, Scrollable, Space, Text};

pub(super) struct Book {
    entries: Vec<Box<dyn EntryEdit>>,
    current: usize,
    scroll_state: scrollable::State,
}

impl Book {
    fn update_current(&mut self) {
        if !self
            .entries
            .get(self.current)
            .map(|e| e.has_focus())
            .unwrap_or(false)
        {
            if let Some((index, _)) = self.entries.iter().enumerate().find(|(_, e)| e.has_focus()) {
                self.current = index;
            } else if self.current >= self.entries.len() {
                self.current = self.entries.len() - 1;
            }
        }
    }
}

impl MainView for Book {
    fn new(_settings: &Settings) -> Box<Self> {
        let first = WorkStartEdit::new(0);
        let second = WorkEntryEdit::new(1);
        Box::new(Book {
            entries: vec![first, second],
            current: 0,
            scroll_state: Default::default(),
        })
    }

    fn view(&mut self, settings: &Settings) -> QElement {
        let date = format!("Date: {}", settings.active_date);
        let resolution = format!("Resolution: {} min", settings.resolution.num_minutes());
        let first_row = Row::new()
            .push(Text::new(date))
            .push(Space::with_width(20.into()))
            .push(Text::new(resolution));

        let mut entry_pane = Scrollable::new(&mut self.scroll_state);
        for e in &mut self.entries {
            entry_pane = entry_pane.push(e.show());
        }
        let second_row = Row::new().push(entry_pane);

        Column::new()
            .push(first_row)
            .push(Rule::horizontal(5))
            .push(second_row)
            .into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::UpdateStart { id, input, valid } => self
                .entries
                .get_mut(id)
                .and_then(|e| e.update(UpdateStart { id, input, valid })),
            Message::UpdateEnd { id, input, valid } => self
                .entries
                .get_mut(id)
                .and_then(|e| e.update(UpdateEnd { id, input, valid })),
            Message::UpdateDescription { id, input } => self
                .entries
                .get_mut(id)
                .and_then(|e| e.update(UpdateDescription { id, input })),
            msg @ (Message::Next | Message::Previous) => {
                self.update_current();
                self.entries
                    .get_mut(self.current)
                    .and_then(|e| e.update(msg))
            }

            m => {
                eprintln!("Unhandled: {:?}", m);
                None
            }
        }
    }
}
