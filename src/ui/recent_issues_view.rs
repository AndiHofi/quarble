use iced_core::Length;
use iced_native::widget::{Column, Row};
use unicode_segmentation::UnicodeSegmentation;

use crate::data::{RecentIssue, RecentIssuesData, RecentIssuesRef};
use crate::ui::util::h_space;
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
    fn view(&self) -> QElement {
        let mut lines = Column::new();
        let mut current_row = Row::new();

        for (num, recent) in self.visible.iter().enumerate().take(8) {
            if num % 2 == 0 {
                let mut tmp = Row::new();
                std::mem::swap(&mut tmp, &mut current_row);
                lines = lines.push(tmp);
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

    let description = limit_text_length(&description);

    Row::with_children(vec![
        text(format!("{}:", num)),
        h_space(style::SPACE),
        text(&recent.issue.ident),
        h_space(style::SPACE),
        text(description),
    ])
    .width(Length::Units(500))
    .into()
}

fn limit_text_length(description: &&str) -> String {
    let description = if let Some((i, _)) = description.grapheme_indices(false).nth(50) {
        format!("{}…", &description[..i])
    } else {
        description.to_string()
    };
    description
}
