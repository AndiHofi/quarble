use crate::conf::SettingsRef;
use crate::data::{Action, ActiveDay, JiraIssue, WorkStart};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::{parse_issue_clipboard, IssueParsed};
use crate::ui::clip_read::ClipRead;
use crate::ui::top_bar::TopBar;
use crate::ui::util::{h_space, v_space};
use crate::ui::{
    day_info_message, style, text, time_info, MainView, Message, QElement, StayActive,
};
use crate::Settings;
use iced_native::widget::{text_input, Column, Row};
use iced_wgpu::TextInput;

#[derive(Clone, Debug)]
pub enum IssueStartMessage {
    TextChanged(String),
}

#[derive(Debug)]
pub struct IssueStartEdit {
    top_bar: TopBar,
    input_state: text_input::State,
    input: String,
    builder: IssueStartBuilder,
    settings: SettingsRef,
    orig: Option<WorkStart>,
}

impl IssueStartEdit {
    pub fn for_active_day(
        settings: SettingsRef,
        active_day: Option<&ActiveDay>,
    ) -> Box<IssueStartEdit> {
        Box::new(Self {
            top_bar: TopBar {
                title: "Start issue:",
                help_text: "[time] [issue] <comment>",
                info: day_info_message(active_day),
                settings: settings.clone(),
            },
            input_state: text_input::State::focused(),
            input: String::new(),
            builder: IssueStartBuilder::default(),
            settings,
            orig: None,
        })
    }

    pub fn entry_to_edit(&mut self, e: WorkStart) {
        let input = format!("{} {} {}", e.ts, e.task.ident, e.description);
        self.builder.parse_input(&self.settings.load(), &input);
        self.input = input;
        self.orig = Some(e);
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
        let value = self.builder.try_build().map(Action::WorkStart);

        if let Some(value) = value {
            if let Some(orig) = std::mem::take(&mut self.orig) {
                Some(Message::ModifyAction {
                    stay_active,
                    orig: Box::new(Action::WorkStart(orig)),
                    update: Box::new(value),
                })
            } else {
                Some(Message::StoreAction(stay_active, value))
            }
        } else {
            None
        }
    }
}

impl MainView for IssueStartEdit {
    fn view(&mut self, _settings: &Settings) -> QElement {
        let input = TextInput::new(&mut self.input_state, "", &self.input, |i| {
            Message::Is(IssueStartMessage::TextChanged(i))
        });

        let info = Row::with_children(vec![
            text("Time:"),
            h_space(style::SPACE),
            time_info(_settings.timeline.time_now(), self.builder.time.clone()),
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
            input.into(),
            v_space(style::SPACE),
            info.into(),
        ])
        .into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Is(IssueStartMessage::TextChanged(input)) => {
                self.builder.parse_input(&self.settings.load(), &input);
                self.input = input;
                self.follow_up()
            }
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

    fn parse_input(&mut self, settings: &Settings, input: &str) {
        let (time, input) = Time::parse_with_offset(&settings.timeline, input);
        let IssueParsed {
            r: issue,
            input: matching,
            rest,
        } = settings.issue_parser.parse_task(input.trim_start());

        self.time = time;

        let rest = rest.trim();
        self.comment = if rest.is_empty() {
            None
        } else {
            Some(rest.to_string())
        };

        let old_issue = std::mem::take(&mut self.issue);
        if matches!(issue, ParseResult::None) {
            if self.issue_input.as_str() != matching {
                self.clipboard = ClipRead::DoRead;
                self.issue = issue;
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
