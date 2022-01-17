use crate::data::{RecentIssue, RecentIssues, RecentIssuesData};
use crate::ui::util::h_space;
use crate::ui::{style, text, MainView, Message, QElement};
use crate::Settings;
use iced_core::Length;
use iced_native::widget::{Column, Row};

pub struct RecentIssuesView {
    recent: RecentIssues,
    filter: String,
    visible: Vec<RecentIssue>,
}

impl RecentIssuesView {
    pub fn create(r: RecentIssues) -> Self {
        let visible: Vec<_> = r.list_recent().to_vec();
        RecentIssuesView {
            recent: r,
            filter: String::new(),
            visible,
        }
    }

    pub fn export_data(&self) -> RecentIssuesData {
        RecentIssuesData {
            issues: self.recent.list_recent().to_vec(),
        }
    }
}

impl MainView for RecentIssuesView {
    fn view(&mut self, _settings: &Settings) -> QElement {
        let mut lines = Column::new();
        let mut current_row = Row::new();

        for (num, recent) in self.visible.iter().enumerate() {
            if num % 3 == 0 {
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
        match msg {
            Message::IssueInput(input) => {
                self.filter = input;
            }
            Message::IssueUsed(issue) => {
                self.recent.issue_used(&issue);
            }
            _ => (),
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
        .unwrap_or("<no description>")
        .to_string();

    Row::with_children(vec![
        text(format!("{}:", num)),
        h_space(style::SPACE),
        text(&recent.issue.ident),
        text(description),
    ])
    .max_width(300)
    .width(Length::Units(300))
    .into()
}
