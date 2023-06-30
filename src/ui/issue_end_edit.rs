use futures::StreamExt;
use iced_core::Length;
use iced_native::widget::text_input::Id;
use iced_native::widget::{text_input, Column, Row};

use crate::conf::SettingsRef;
use crate::data::{ActiveDay, JiraIssue, WorkEnd};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::{IssueParsed, IssueParser};
use crate::ui::book_single::nparsing::WTime;
use crate::ui::my_text_input::MyTextInput;
use crate::ui::single_edit_ui::{FocusableUi, SingleEditUi};
use crate::ui::top_bar::TopBar;
use crate::ui::util::{h_space, v_space};
use crate::ui::{day_info_message, style, text, time_info, MainView, Message, QElement};

#[derive(Clone, Debug)]
pub enum IssueEndMessage {
    InputChanged(String),
}

pub struct IssueEndEdit {
    top_bar: TopBar,
    end_time: MyTextInput,
    issue_id: MyTextInput,
    message: MyTextInput,
    description: MyTextInput,
    time: ParseResult<WTime, ()>,
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
        let default_issue = active_day
            .as_ref()
            .and_then(|d| d.current_issue(guard.timeline.time_now()));

        let issue_id_text = default_issue
            .as_ref()
            .map(|e| e.ident.as_str())
            .unwrap_or_default();
        let description_text = default_issue
            .as_ref()
            .and_then(|e| e.description.as_ref())
            .map(|e| e.as_str())
            .unwrap_or_default();
        Box::new(Self {
            top_bar: TopBar {
                title: "End issue:",
                help_text: "[<time>] [<issue_id>]",
                info: day_info_message(active_day),
                settings: settings.clone(),
            },
            end_time: MyTextInput::new("", |_| true).with_placeholder("end time"),
            issue_id: MyTextInput::new(issue_id_text, |_| true).with_placeholder("issue id"),
            message: MyTextInput::new(description_text, |_| true).with_placeholder("message"),
            description: MyTextInput::new(description_text, |_| true)
                .with_placeholder("description"),
            time: ParseResult::Valid(WTime::Time(guard.timeline.time_now())),
            settings,
            issue: ParseResult::None,
            default_issue,
            orig: None,
        })
    }
}

impl SingleEditUi<WorkEnd> for IssueEndEdit {
    fn as_text(&self, e: &WorkEnd) -> String {
        format!("{} {}", e.ts, e.task.ident)
    }

    fn set_orig(&mut self, orig: WorkEnd) {
        let input = self.as_text(&orig);
        self.orig = Some(orig);
        self.update_default_input(input);
    }

    fn try_build(&self) -> Option<WorkEnd> {
        let issue = match &self.issue {
            ParseResult::None => self.default_issue.clone(),
            ParseResult::Valid(i) => Some(i.clone()),
            _ => None,
        };

        match (issue, self.time.as_ref()) {
            (Some(task), ParseResult::Valid(WTime::Time(time))) => {
                let action = WorkEnd { task, ts: *time };
                Some(action)
            }
            _ => None,
        }
    }

    fn update_input(&mut self, id: text_input::Id, input: String) -> Option<Message> {
        consume_input(
            id,
            input,
            &mut [
                &mut self.end_time,
                &mut self.issue_id,
                &mut self.message,
                &mut self.description,
            ],
        );
        let settings = self.settings.load();
        // let recent_issues = self.recent_issues.borrow();
        // self.builder.parse_input(&settings, self.last_end, &recent_issues, &self.start.text, &self.id.text, &self.comment.text, &self.description.text);
        // self.follow_up
        None
    }
}

impl FocusableUi for IssueEndEdit {
    fn default_focus(&self) -> Id {
        self.end_time.id.clone()
    }
}

impl MainView for IssueEndEdit {
    fn view(&self) -> QElement {
        let input = Row::with_children(vec![
            self.end_time.show_text_input(Length::Units(200)).into(),
            self.issue_id.show_text_input(Length::Units(300)).into(),
            self.description.show_text_input(Length::Fill).into(),
        ])
        .spacing(style::SPACE_PX);

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
            Message::Input(id, input) => self.update_input(id, input),
            Message::SubmitCurrent(stay_active) => {
                Self::on_submit_message(self.try_build(), &mut self.orig, stay_active)
            }
            Message::StoreSuccess(stay_active) => stay_active.on_main_view_store(),
            _ => None,
        }
    }
}

pub(super) fn consume_input(
    id: text_input::Id,
    input: String,
    fields: &mut [&mut MyTextInput],
) -> Option<Message> {
    fields
        .iter_mut()
        .find(|e| e.id == id)
        .and_then(|f| f.accept_input(input))
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
