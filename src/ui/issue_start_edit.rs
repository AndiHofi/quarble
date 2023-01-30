use iced_native::widget::{text_input, Column, Row};

use crate::conf::SettingsRef;
use crate::data::{ActiveDay, JiraIssue, RecentIssues, RecentIssuesRef, WorkStart};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::{parse_issue_clipboard, IssueParsed, IssueParser, IssueParserWithRecent};
use crate::ui::clip_read::ClipRead;
use crate::ui::my_text_input::MyTextInput;
use crate::ui::single_edit_ui::{FocusableUi, SingleEditUi};
use crate::ui::stay_active::StayActive;
use crate::ui::top_bar::TopBar;
use crate::ui::util::{h_space, v_space};
use crate::ui::{day_info_message, style, text, time_info, MainView, Message, QElement};
use crate::Settings;

#[derive(Clone, Debug)]
pub enum IssueStartMessage {
    TextChanged(String),
}

pub struct IssueStartEdit {
    top_bar: TopBar,
    input: MyTextInput,
    builder: IssueStartBuilder,
    settings: SettingsRef,
    orig: Option<WorkStart>,
    last_end: Option<Time>,
    recent_issues: RecentIssuesRef,
}

const INPUT_ID: &str = "ISE01";

impl IssueStartEdit {
    pub fn for_active_day(
        settings: SettingsRef,
        recent_issues: RecentIssuesRef,
        active_day: Option<&ActiveDay>,
    ) -> Box<IssueStartEdit> {
        let now = settings.load().timeline.time_now();
        let last_end = active_day.and_then(|d| d.last_action_end(now));
        Box::new(Self {
            top_bar: TopBar {
                title: "Start issue:",
                help_text: "[time] [issue] <comment>",
                info: day_info_message(active_day),
                settings: settings.clone(),
            },
            input: MyTextInput::new("", |_| true),
            builder: IssueStartBuilder::default(),
            settings,
            orig: None,
            last_end,
            recent_issues,
        })
    }

    fn follow_up(&mut self) -> Option<Message> {
        if matches!(self.builder.clipboard, ClipRead::DoRead) {
            self.builder.clipboard = ClipRead::Reading;
            Some(Message::ReadClipboard)
        } else {
            None
        }
    }

    fn on_submit(&mut self, stay_active: StayActive) -> Option<Message> {
        let value = self.builder.try_build();

        Self::on_submit_message(value, &mut self.orig, stay_active)
    }
}

impl SingleEditUi<WorkStart> for IssueStartEdit {
    fn as_text(&self, e: &WorkStart) -> String {
        format!("{} {} {}", e.ts, e.task.ident, e.description)
    }

    fn set_orig(&mut self, orig: WorkStart) {
        let input = self.as_text(&orig);
        self.orig = Some(orig);
        self.update_default_input(input);
    }

    fn try_build(&self) -> Option<WorkStart> {
        self.builder.try_build()
    }

    fn update_input(&mut self, _id: text_input::Id, input: String) -> Option<Message> {
        self.input.text = input;
        let x = self.settings.load();
        self.builder.parse_input(
            &x,
            self.last_end,
            &self.recent_issues.borrow(),
            &self.input.text,
        );

        self.follow_up()
    }
}

impl FocusableUi for IssueStartEdit {
    fn default_focus(&self) -> text_input::Id {
        self.input.id.clone()
    }
}

impl MainView for IssueStartEdit {
    fn view(&self) -> QElement {
        let input = self.input.show("");
        let settings = self.settings.load();

        let info = Row::with_children(vec![
            text("Time:"),
            h_space(style::SPACE),
            time_info(settings.timeline.time_now(), self.builder.time.clone()),
            h_space(style::DSPACE),
            text("Issue:"),
            h_space(style::SPACE),
            text(
                self.builder
                    .issue
                    .get_ref()
                    .map(|i| i.ident.as_str())
                    .unwrap_or("<none>"),
            ),
            h_space(style::DSPACE),
            text("Comment:"),
            h_space(style::SPACE),
            text(self.builder.comment.as_deref().unwrap_or("<none>")),
        ]);
        Column::with_children(vec![
            self.top_bar.view(),
            v_space(style::SPACE),
            input,
            v_space(style::SPACE),
            info.into(),
        ])
        .into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Input(id, input) => self.update_input(id, input),
            Message::ClipboardValue(value) => {
                self.builder.apply_clipboard(value);
                None
            }
            Message::SubmitCurrent(stay_active) => self.on_submit(stay_active),
            Message::StoreSuccess(stay_active) => stay_active.on_main_view_store(),
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
struct IssueStartBuilder {
    time: ParseResult<Time, ()>,
    issue: ParseResult<JiraIssue, ()>,
    clipboard: ClipRead,
    issue_input: String,
    comment: Option<String>,
}

impl IssueStartBuilder {
    fn try_build(&self) -> Option<WorkStart> {
        match (&self.time, &self.issue, &self.comment) {
            (ParseResult::Valid(time), ParseResult::Valid(i), Some(c)) => Some(WorkStart {
                ts: *time,
                task: i.clone(),
                description: c.to_string(),
            }),
            _ => None,
        }
    }

    fn parse_input(
        &mut self,
        settings: &Settings,
        last_end: Option<Time>,
        recent_issues: &RecentIssues,
        input: &str,
    ) {
        let (time, input) = if let Some(rest) = input.strip_prefix('l') {
            (
                last_end
                    .map(ParseResult::Valid)
                    .unwrap_or(ParseResult::Invalid(())),
                rest,
            )
        } else {
            Time::parse_with_offset(&settings.timeline, input)
        };

        let parser = IssueParserWithRecent::new(&settings.issue_parser, recent_issues);

        let IssueParsed {
            r: issue,
            input: matching,
            rest,
            is_recent: _,
        } = parser.parse_task(input.trim_start());

        self.time = time;

        let rest = rest.trim();
        self.comment = if rest.is_empty() {
            issue.get_ref().and_then(|e| e.default_action.clone())
        } else {
            Some(rest.to_string())
        };

        let old_issue = std::mem::take(&mut self.issue);
        if matches!(issue, ParseResult::None) {
            if self.issue_input.as_str() != matching {
                self.clipboard = ClipRead::DoRead;
                self.issue = ParseResult::None;
            } else {
                self.issue = old_issue;
            }
        } else {
            self.issue = issue;
        }
        self.issue_input = matching.to_string();
    }

    fn apply_clipboard(&mut self, value: Option<String>) {
        self.clipboard = ClipRead::None;
        if let ParseResult::None = self.issue {
            let value = value.as_deref().unwrap_or("");
            if !value.is_empty() {
                if let Some(ji) = parse_issue_clipboard(value) {
                    self.issue = ParseResult::Valid(ji);
                } else {
                    self.issue = ParseResult::Invalid(());
                    self.clipboard = ClipRead::Invalid;
                }
            } else {
                self.issue = ParseResult::Invalid(());
                self.clipboard = ClipRead::NoClip
            }
        } else {
            eprintln!("Cannot apply clipboard");
            self.clipboard = ClipRead::Unexpected;
        }
    }
}

#[cfg(test)]
mod test {
    use crate::conf::SettingsRef;
    use crate::data::test_support::time;
    use crate::data::{ActiveDay, JiraIssue, Location, RecentIssuesRef, WorkStart};
    use crate::ui::issue_start_edit::IssueStartEdit;
    use crate::ui::single_edit_ui::SingleEditUi;
    use crate::util::{StaticTimeline, Timeline};
    use crate::Settings;

    #[test]
    fn parse_with_recent() {
        let (_, recent, mut ui) = build_ui();
        let result = ui.convert_input("11 r1");

        assert_eq!(
            result,
            Some(WorkStart {
                ts: time("11"),
                task: recent.get(0).issue,
                description: "default action".to_string()
            })
        );
    }

    #[test]
    fn parse_with_recent_override_action() {
        let (_, recent, mut ui) = build_ui();

        let result = ui.convert_input("11 r1 changed action");
        assert_eq!(
            result,
            Some(WorkStart {
                ts: time("11"),
                task: recent.get(0).issue,
                description: "changed action".to_string()
            })
        )
    }

    fn build_ui() -> (SettingsRef, RecentIssuesRef, Box<IssueStartEdit>) {
        let timeline: Timeline = StaticTimeline::parse("2022-01-20 10:15").into();
        let settings = Settings {
            timeline: timeline.clone(),
            ..Settings::default()
        }
        .into_settings_ref();

        let recent = RecentIssuesRef::empty(settings.clone());
        let issue = JiraIssue {
            ident: "RECENT-123".to_string(),
            default_action: Some("default action".to_string()),
            description: None,
        };
        recent.issue_used_with_comment(&issue, None);

        let ui = IssueStartEdit::for_active_day(
            settings.clone(),
            recent.clone(),
            Some(&ActiveDay::new(timeline.today(), Location::Office, None)),
        );

        (settings, recent, ui)
    }
}
