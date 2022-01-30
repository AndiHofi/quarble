use iced_native::widget::{text_input, Column, Row};
use iced_wgpu::TextInput;

use crate::conf::SettingsRef;
use crate::data::{ActiveDay, JiraIssue, WorkEnd};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::{IssueParsed, IssueParser};
use crate::ui::single_edit_ui::SingleEditUi;
use crate::ui::top_bar::TopBar;
use crate::ui::util::{h_space, v_space};
use crate::ui::{day_info_message, style, text, time_info, MainView, Message, QElement};
use crate::Settings;

#[derive(Clone, Debug)]
pub enum IssueEndMessage {
    InputChanged(String),
}

#[derive(Debug)]
pub struct IssueEndEdit {
    top_bar: TopBar,
    input_state: text_input::State,
    input: String,
    time: ParseResult<Time, ()>,
    issue: ParseResult<JiraIssue, ()>,
    settings: SettingsRef,
    default_issue: Option<JiraIssue>,
    orig: Option<WorkEnd>,
}

impl IssueEndEdit {
    pub fn for_active_day(
        settings: SettingsRef,
        active_day: Option<&ActiveDay>,
    ) -> Box<IssueEndEdit> {
        let guard = settings.load();
        let default_issue = active_day.and_then(|d| d.current_issue(guard.timeline.time_now()));

        Box::new(Self {
            top_bar: TopBar {
                title: "End issue:",
                help_text: "[<time>] [<issue_id>]",
                info: day_info_message(active_day),
                settings: settings.clone(),
            },
            input_state: text_input::State::focused(),
            input: String::new(),
            time: ParseResult::Valid(guard.timeline.time_now()),
            settings,
            issue: ParseResult::None,
            default_issue,
            orig: None,
        })
    }
}

impl SingleEditUi<WorkEnd> for IssueEndEdit {
    fn update_input(&mut self, input: String) {
        self.input = input;
        let guard = self.settings.load();
        let (time, input) = Time::parse_with_offset(&guard.timeline, &self.input);
        self.time = time;
        let IssueParsed { r, rest, .. } = guard.issue_parser.parse_task(input.trim_start());
        if rest.is_empty() {
            self.issue = r;
        } else {
            self.issue = ParseResult::Invalid(());
        }
    }

    fn as_text(&self, e: &WorkEnd) -> String {
        format!("{} {}", e.ts, e.task.ident)
    }

    fn set_orig(&mut self, orig: WorkEnd) {
        self.orig = Some(orig)
    }

    fn try_build(&self) -> Option<WorkEnd> {
        let issue = match &self.issue {
            ParseResult::None => self.default_issue.clone(),
            ParseResult::Valid(i) => Some(i.clone()),
            _ => None,
        };

        match (issue, self.time.as_ref()) {
            (Some(task), ParseResult::Valid(time)) => {
                let action = WorkEnd { task, ts: *time };
                Some(action)
            }
            _ => None,
        }
    }
}

impl MainView for IssueEndEdit {
    fn view(&mut self, _settings: &Settings) -> QElement {
        let input = TextInput::new(&mut self.input_state, "now", &self.input, |e| {
            Message::Ie(IssueEndMessage::InputChanged(e))
        });

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
            time_info(self.settings.load().timeline.time_now(), self.time.clone()),
            h_space(style::DSPACE),
            text("Issue:"),
            h_space(style::SPACE),
            text(issue_text),
        ]);

        Column::with_children(vec![
            self.top_bar.view(),
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
                self.update_input(text);
                None
            }
            Message::SubmitCurrent(stay_active) => {
                Self::on_submit_message(self.try_build(), &mut self.orig, stay_active)
            }
            Message::StoreSuccess(stay_active) => stay_active.on_main_view_store(),
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::conf::into_settings_ref;
    use crate::ui::issue_end_edit::{IssueEndEdit, IssueEndMessage};
    use crate::ui::stay_active::StayActive;
    use crate::ui::{MainView, Message};
    use crate::util::StaticTimeline;
    use crate::Settings;

    #[test]
    fn test_issue_end() {
        let timeline = StaticTimeline::parse("2022-1-10 17:00").into();
        let settings = into_settings_ref(Settings {
            timeline,
            ..Settings::default()
        });
        let mut ui = IssueEndEdit::for_active_day(settings, None);

        let on_input = ui.update(Message::Ie(IssueEndMessage::InputChanged(
            "+0 QU-42".to_string(),
        )));
        assert!(matches!(on_input, None));

        let on_submit = ui.update(Message::SubmitCurrent(StayActive::Yes));
        assert!(matches!(
            on_submit,
            Some(Message::StoreAction(
                StayActive::Yes,
                crate::data::Action::WorkEnd(_)
            ))
        ));
    }
}
