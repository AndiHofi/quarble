use crate::conf::Settings;
use crate::data::{Action, ActiveDay};
use crate::parsing::time::Time;
use crate::ui::util::h_space;
use crate::ui::{style, text};
use crate::ui::{MainView, Message, QElement};
use iced_core::alignment::Horizontal;
use iced_core::Length;
use iced_winit::widget::{scrollable, Column, Container, Row, Scrollable, Space, Text};

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

        Column::with_children(vec![
            Text::new(day).into(),
            Space::with_height(style::SPACE).into(),
            Text::new(active_issue).into(),
            Space::with_height(style::SPACE).into(),
            Container::new(entries_scroll)
                .style(content_style)
                .padding(style::SPACE_PX)
                .into(),
        ])
        .into()
    }

    fn update(&mut self, _msg: Message) -> Option<Message> {
        None
    }
}
