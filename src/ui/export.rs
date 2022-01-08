use crate::conf::SettingsRef;
use crate::data::{Action, ActiveDay, NormalizedDay, Normalizer, TimeCockpitExporter};
use crate::ui::util::{h_space, v_space};
use crate::ui::{style, text, MainView, Message, QElement};
use crate::Settings;
use iced_core::Length;
use iced_native::widget::{
    button, scrollable, Button, Checkbox, Column, Container, Row, Scrollable,
};
use std::num::NonZeroU32;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum DayExportMessage {
    ChangeNormalize(bool),
    TriggerExport,
}

pub struct DayExportUi {
    active_day: Option<ActiveDay>,
    normalized: Option<NormalizedDay>,
    actions: Vec<Action>,
    export_text: Option<Arc<String>>,
    msg: Option<String>,
    clip_button: button::State,
    settings: SettingsRef,
    combine_bookings: bool,
    add_break: bool,
    scroll_state: scrollable::State,
}

impl DayExportUi {
    pub fn for_active_day(settings: SettingsRef, current_day: Option<&ActiveDay>) -> Box<Self> {
        let combine_bookings = true;
        let add_break = true;

        let mut ui = Box::new(Self {
            active_day: current_day.cloned(),
            normalized: None,
            actions: Vec::new(),
            export_text: None,
            msg: None,
            clip_button: button::State::new(),
            settings,
            combine_bookings,
            add_break,
            scroll_state: scrollable::State::new(),
        });

        ui.normalize_day();

        ui
    }

    fn normalize_day(&mut self) {
        let s = self.settings.load();
        let (normalized, actions, error) = if let Some(current_day) = self.active_day.as_ref() {
            let n = Normalizer {
                resolution: NonZeroU32::new(s.resolution.num_minutes() as u32)
                    .unwrap_or_else(|| NonZeroU32::new(1).unwrap()),
                breaks_config: s.breaks.clone(),
                combine_bookings: self.combine_bookings,
                add_break: self.add_break,
            }
            .create_normalized(current_day);

            match n {
                Ok(n) => {
                    let actions = n.entries.iter().cloned().map(Action::Work).collect();
                    (Some(n), actions, None)
                }
                Err(e) => (None, Vec::new(), Some(e)),
            }
        } else {
            (None, Vec::new(), None)
        };

        let export_text = normalized
            .as_ref()
            .map(|w| Arc::new(TimeCockpitExporter::export(w)));

        self.normalized = normalized;
        self.actions = actions;
        self.msg = error;
        self.export_text = export_text;
    }
}

impl MainView for DayExportUi {
    fn view(&mut self, _settings: &Settings) -> QElement {
        let title_text = self
            .active_day
            .as_ref()
            .map(|a| a.get_day().to_string())
            .unwrap_or_else(|| "No active day".to_string());

        let top_row = Row::with_children(vec![
            text(format!("Export: {}", title_text)),
            h_space(style::DSPACE),
            text(self.msg.as_deref().unwrap_or("Export with <ctrl>+C")),
        ]);

        let mut scroll = Scrollable::new(&mut self.scroll_state).width(Length::Fill);
        for e in self.actions.iter().map(super::current_day::action_row) {
            scroll = scroll.push(e);
        }

        let scroll = Container::new(scroll)
            .style(style::ContentStyle)
            .width(Length::Fill)
            .height(Length::Fill);
        let buttons = Column::with_children(vec![
            Button::new(&mut self.clip_button, text("Copy"))
                .on_press(Message::Export(DayExportMessage::TriggerExport))
                .into(),
            v_space(style::DSPACE),
            Checkbox::new(self.combine_bookings, "Combine", |b| {
                Message::Export(DayExportMessage::ChangeNormalize(b))
            })
            .into(),
        ])
        .width(Length::Units(200));

        let body = Row::with_children(vec![scroll.into(), h_space(style::SPACE), buttons.into()]);

        Column::with_children(vec![top_row.into(), v_space(style::SPACE), body.into()]).into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Export(DayExportMessage::ChangeNormalize(combine)) => {
                self.combine_bookings = combine;
                self.normalize_day();
                None
            }
            Message::Export(DayExportMessage::TriggerExport) => match self.export_text {
                Some(ref t) => {
                    let entries = t.lines().count();
                    self.msg = Some(format!("exported {} entries", entries));
                    Some(Message::WriteClipboard(t.clone()))
                }
                None => {
                    self.msg = Some("Nothing to export".to_string());
                    None
                }
            },
            _ => None,
        }
    }
}
