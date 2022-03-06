use iced_core::Length;
use iced_native::widget::{Column, Row, Text};
use unicode_segmentation::UnicodeSegmentation;

use crate::data::{RecentIssue, RecentIssuesData, RecentIssuesRef};
use crate::ui::util::{h_space, v_space};
use crate::ui::{style, text, MainView, Message, QElement};

pub struct RecentIssuesView {
    recent: RecentIssuesRef,
    filter: String,
    visible: Vec<RecentIssue>,
}

impl RecentIssuesView {
    pub fn create(r: RecentIssuesRef) -> Self {
        let guard = r.borrow();
        let visible: Vec<_> = guard.list_recent().to_vec();
        RecentIssuesView {
            recent: r,
            filter: String::new(),
            visible,
        }
    }

    pub fn export_data(&self) -> RecentIssuesData {
        RecentIssuesData {
            issues: self.recent.borrow().list_recent().to_vec(),
        }
    }

    pub fn refresh(&mut self) {
        self.update_filter(String::new())
    }

    fn update_filter(&mut self, input: String) {
        self.filter = input;
        let guard = self.recent.borrow();
        if self.filter.trim().is_empty() {
            self.visible = guard.list_recent().to_vec();
        } else {
            let input = self.filter.as_str();
            self.visible = guard
                .list_recent()
                .iter()
                .filter(|e| {
                    e.issue.ident.contains(input)
                        || e.issue
                            .description
                            .as_deref()
                            .filter(|d| d.contains(input))
                            .is_some()
                })
                .cloned()
                .collect();
        }
    }
}

impl MainView for RecentIssuesView {
    fn view(&mut self) -> QElement {
        let mut lines = Column::new();
        let mut current_row = Row::new();

        for (num, recent) in self.visible.iter().enumerate().take(20) {
            if num % 2 == 0 && num != 0 {
                let mut tmp = Row::new();
                std::mem::swap(&mut tmp, &mut current_row);
                lines = lines.push(tmp);
                lines = lines.push(v_space(Length::Units(3)));
            }
            current_row = current_row.push(build_recent(num + 1, recent));
        }
        lines = lines.push(current_row);

        lines.into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        if let Message::IssueInput(input) = msg {
            self.filter = input;
        };
        None
    }
}

fn build_recent(num: usize, recent: &RecentIssue) -> QElement {
    let description = recent
        .issue
        .description
        .as_deref()
        .or(recent.issue.default_action.as_deref())
        .unwrap_or("<no description>");

    let action = recent.issue.default_action.as_deref().unwrap_or("-");

    let description = limit_text_length(description, 55);
    let action = limit_text_length(action, 25);

    Row::with_children(vec![
        Text::new(format!("{}:", num))
            .width(Length::Units(22))
            .into(),
        h_space(style::SPACE),
        Text::new(&recent.issue.ident)
            .width(Length::Units(100))
            .into(),
        h_space(style::SPACE),
        Text::new(action).width(Length::Units(190)).into(),
        h_space(style::SPACE),
        Text::new(description).into(),
    ])
    .max_width(800)
    .width(Length::Units(800))
    .into()
}

fn limit_text_length(description: &str, length: usize) -> String {
    let description = if let Some((i, _)) = description.grapheme_indices(false).nth(length) {
        format!("{}…", &description[..i])
    } else {
        description.to_string()
    };
    description
}
