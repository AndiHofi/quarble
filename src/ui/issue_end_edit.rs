use crate::data::{Action, ActiveDay, JiraIssue, WorkEnd};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::IssueParsed;
use crate::ui::util::{h_space, v_space};
use crate::ui::{input_message, style, text, time_info, MainView, Message, QElement};
use crate::Settings;
use iced_native::widget::{text_input, Column, Row};
use iced_wgpu::TextInput;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum IssueEndMessage {
    InputChanged(String),
}

#[derive(Debug)]
pub struct IssueEndEdit {
    input_state: text_input::State,
    input: String,
    input_message: String,
    time: ParseResult<Time, ()>,
    issue: ParseResult<JiraIssue, ()>,
    settings: Arc<Settings>,
    default_issue: Option<JiraIssue>,
}

impl IssueEndEdit {
    pub fn for_active_day(
        settings: Arc<Settings>,
        active_day: Option<&ActiveDay>,
    ) -> Box<IssueEndEdit> {
        let default_issue = active_day
            .and_then(|d| d.current_issue(settings.timeline.time_now()))
            .map(JiraIssue::clone);

        Box::new(Self {
            input_state: text_input::State::focused(),
            input: String::new(),
            input_message: input_message(
                "End issue",
                active_day
                    .map(|d| d.actions())
                    .unwrap_or(ActiveDay::no_action()),
            ),
            time: ParseResult::Valid(settings.timeline.time_now()),
            settings,
            issue: ParseResult::None,
            default_issue,
        })
    }

    fn parse_input(&mut self, input: &str) {
        let (time, input) = Time::parse_with_offset(&self.settings.timeline, input);
        self.time = time;
        let IssueParsed { r, rest, .. } = self.settings.issue_parser.parse_task(input.trim_start());
        if rest.is_empty() {
            self.issue = r;
        } else {
            self.issue = ParseResult::Invalid(());
        }
    }

    fn on_submit_message(&self) -> Message {
        let issue = match &self.issue {
            ParseResult::None => self.default_issue.clone(),
            ParseResult::Valid(i) => Some(i.clone()),
            _ => None,
        };

        match (issue, self.time.as_ref()) {
            (Some(task), ParseResult::Valid(time)) => {
                let action = WorkEnd { task, ts: *time };
                Message::StoreAction(Action::WorkEnd(action))
            }
            _ => Message::Update,
        }
    }
}

impl MainView for IssueEndEdit {
    fn view(&mut self, _settings: &Settings) -> QElement {
        let on_submit = self.on_submit_message();
        let input = TextInput::new(&mut self.input_state, "now", &self.input, |e| {
            Message::Ie(IssueEndMessage::InputChanged(e))
        })
        .on_submit(on_submit);

        let issue_text: String = if let ParseResult::Valid(i) = &self.issue {
            i.ident.clone()
        } else if let ParseResult::Invalid(_) = &self.issue {
            "<invalid>".to_string()
        } else if let Some(i) = &self.default_issue {
            format!("Active {}", i.ident)
        } else {
            "<none>".to_string()
        };

        let info = Row::with_children(vec![
            text("Time:"),
            h_space(style::SPACE),
            time_info(self.settings.timeline.time_now(), self.time.clone()),
            h_space(style::DSPACE),
            text("Issue:"),
            h_space(style::SPACE),
            text(issue_text),
        ]);

        Column::with_children(vec![
            text(&self.input_message),
            v_space(style::SPACE),
            input.into(),
            v_space(style::SPACE),
            info.into(),
        ])
        .into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Ie(IssueEndMessage::InputChanged(text)) => {
                self.parse_input(&text);
                self.input = text;
                None
            }
            Message::StoreSuccess => Some(Message::Exit),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ui::issue_end_edit::{IssueEndEdit, IssueEndMessage};
    use crate::ui::{MainView, Message};
    use crate::util::StaticTimeline;
    use crate::Settings;
    use std::sync::Arc;

    #[test]
    fn test_issue_end() {
        let timeline = StaticTimeline::parse("2022-1-10 17:00");
        let settings = Arc::new(Settings::default().with_timeline(timeline));
        let mut ui = IssueEndEdit::for_active_day(settings.clone(), None);

        let on_input = ui.update(Message::Ie(IssueEndMessage::InputChanged(
            "+0 QU-42".to_string(),
        )));
        assert!(matches!(on_input, None));

        let on_submit = ui.on_submit_message();
        assert!(matches!(
            on_submit,
            Message::StoreAction(crate::data::Action::WorkEnd(_))
        ));
    }
}
