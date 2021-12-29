use crate::conf::Settings;
use crate::data::ActiveDay;
use crate::ui::style;
use crate::ui::{MainView, Message, QElement};
use iced_winit::widget::{scrollable, Column, Container, Scrollable, Space, Text};

#[derive(Debug, Clone)]
pub struct CurrentDayUI {
    data: ActiveDay,
    scroll_state: scrollable::State,
}

impl CurrentDayUI {
    pub fn for_active_day(d: Option<&ActiveDay>) -> Box<Self> {
        Box::new(Self {
            data: match d {
                Some(d) => d.clone(),
                None => ActiveDay::default(),
            },
            scroll_state: Default::default(),
        })
    }
}

impl MainView for CurrentDayUI {
    fn new(_settings: &Settings) -> Box<Self> {
        Box::new(Self {
            data: ActiveDay::default(),
            scroll_state: Default::default(),
        })
    }

    fn view<'a>(&'a mut self, _settings: &Settings) -> QElement<'a> {
        let day = self.data.get_day().to_string();
        let active_issue = self
            .data
            .active_issue()
            .map(|i| i.to_string())
            .unwrap_or_else(|| "No active issue".to_string());

        let entries: Vec<QElement<'a>> = self
            .data
            .actions()
            .iter()
            .map(|e| format!("{:?}", e))
            .map(|e| {
                Container::new(Text::new(e))
                    .padding(style::WINDOW_PADDING)
                    .into()
            })
            .collect();
        let mut entries_scroll = Scrollable::new(&mut self.scroll_state);
        for e in entries {
            entries_scroll = entries_scroll.push(e);
        }

        Column::with_children(vec![
            Text::new(day).into(),
            Space::with_height(style::SPACE).into(),
            Text::new(active_issue).into(),
            Space::with_height(style::SPACE).into(),
            entries_scroll.into(),
        ])
        .into()
    }

    fn update(&mut self, _msg: Message) -> Option<Message> {
        None
    }
}
