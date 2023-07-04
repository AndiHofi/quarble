use iced_core::Length;
use iced_native::widget::{text_input, Column, Row};

use crate::conf::SettingsRef;
use crate::data::{ActiveDay, JiraIssue, RecentIssues, RecentIssuesRef, WorkStart};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::{parse_issue_clipboard, IssueParser, IssueParserWithRecent};
use crate::ui::book_single::nparsing;
use crate::ui::book_single::nparsing::WTime;
use crate::ui::clip_read::ClipRead;
use crate::ui::my_text_input::MyTextInput;
use crate::ui::single_edit_ui::{FocusableUi, SingleEditUi};
use crate::ui::stay_active::StayActive;
use crate::ui::top_bar::TopBar;
use crate::ui::util::{consume_input, h_space, v_space};
use crate::ui::{day_info_message, style, text, time_info, MainView, Message, QElement};
use crate::Settings;

#[derive(Clone, Debug)]
pub enum IssueStartMessage {
    TextChanged(String),
}

pub struct IssueStartEdit {
    top_bar: TopBar,
    start: MyTextInput,
    id: MyTextInput,
    comment: MyTextInput,
    description: MyTextInput,
    builder: IssueStartBuilder,
    settings: SettingsRef,
    orig: Option<WorkStart>,
    last_end: Option<Time>,
    recent_issues: RecentIssuesRef,
    has_input: Option<text_input::Id>,
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
            start: MyTextInput::msg_aware("", nparsing::time_input).with_placeholder("start"),
            id: MyTextInput::msg_aware("", nparsing::issue_input).with_placeholder("key"),
            comment: MyTextInput::new("", |_| true).with_placeholder("comment"),
            description: MyTextInput::new("", |_| true).with_placeholder("description"),
            builder: IssueStartBuilder::default(),
            settings,
            orig: None,
            last_end,
            recent_issues,
            has_input: None,
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

    //noinspection DuplicatedCode
    fn update_input(&mut self, id: text_input::Id, input: String) -> Option<Message> {
        let f = &self.has_input;
        let text_follow_up = if self.start.id == id {
            self.start.accept_input(input)
        } else if self.id.id == id {
            self.id.accept_input(input)
        } else if self.comment.id == id {
            self.comment.accept_input(input)
        } else if self.description.id == id {
            self.description.accept_input(input)
        } else {
            None
        };

        if text_follow_up.is_some() {
            return text_follow_up;
        }

        if self.id.is_focused(f) || self.comment.id == id {
            return Some(Message::FilterRecent(
                self.id.text.as_str().into(),
                self.comment.text.as_str().into(),
            ));
        }

        let settings = self.settings.load();
        let recent_issues = self.recent_issues.borrow();
        self.builder.parse_input(
            &settings,
            self.last_end,
            &recent_issues,
            &self.start.text,
            &self.id.text,
            &self.comment.text,
            &self.description.text,
        );
        self.follow_up()
    }
}

impl FocusableUi for IssueStartEdit {
    fn default_focus(&self) -> text_input::Id {
        self.start.id.clone()
    }
}

impl MainView for IssueStartEdit {
    fn view(&self) -> QElement {
        let input_row: Vec<QElement> = vec![
            self.start.show_text_input(Length::Units(100)).into(),
            self.id.show_text_input(Length::Units(300)).into(),
            self.comment.show_text_input(Length::Fill).into(),
            self.description.show_text_input(Length::Units(200)).into(),
        ];
        let input_row = Row::with_children(input_row).spacing(style::SPACE_PX);
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
            input_row.into(),
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
            Message::Focus(id) => {
                self.has_input = Some(id);
                None
            }
            _ => None,
        }
    }
}

#[derive(Debug, Default)]
struct IssueStartBuilder {
    time: ParseResult<WTime, ()>,
    issue: ParseResult<JiraIssue, ()>,
    clipboard: ClipRead,
    comment: Option<String>,
}

impl IssueStartBuilder {
    fn try_build(&self) -> Option<WorkStart> {
        match (&self.time, &self.issue, &self.comment) {
            (ParseResult::Valid(WTime::Time(time)), ParseResult::Valid(i), Some(c)) => {
                Some(WorkStart {
                    ts: *time,
                    task: i.clone(),
                    description: c.to_string(),
                })
            }
            _ => None,
        }
    }

    fn parse_input(
        &mut self,
        settings: &Settings,
        last_end: Option<Time>,
        recent_issues: &RecentIssues,
        start: &str,
        key: &str,
        comment: &str,
        description: &str,
    ) {
        let start = super::book_single::nparsing::parse_start(start, &settings.timeline);

        let parser = IssueParserWithRecent::new(&settings.issue_parser, recent_issues);
        let issue = parser.parse_issue_key(key.trim());

        self.time = start;

        self.comment = if comment.is_empty() {
            issue.get_ref().and_then(|e| e.default_action.clone())
        } else {
            Some(comment.to_string())
        };

        let old_issue = std::mem::take(&mut self.issue);
        if matches!(issue, ParseResult::None) {
            self.issue = old_issue;
        } else {
            self.issue = issue;
        }
        dbg!(self);
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
