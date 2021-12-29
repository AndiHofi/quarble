use crate::conf::Settings;
use crate::data::{ActiveDay, JiraIssue, Task, Work};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::{parse_input, parse_input_rel};
use crate::ui::util::{h_space, v_space};
use crate::ui::{input_message, style, MainView, Message, QElement};
use crate::util::Timeline;
use crate::DefaultTimeline;
use iced_native::widget::Row;
use iced_wgpu::TextInput;
use iced_winit::widget::{text_input, Column, Text};
use lazy_static::lazy_static;
use std::ops::Deref;
use std::sync::Arc;

lazy_static! {
    static ref ISSUE_NUM: regex::Regex =
        regex::RegexBuilder::new(r"(?P<id>(?:[a-zA-Z]+)-(?:[0-9]+))(?:(?:\W)+(?P<comment>.*))?")
            .build()
            .unwrap();
}

#[derive(Clone, Debug)]
pub enum BookSingleMessage {
    TextChanged(String),
}

pub struct BookSingleUI {
    input_state: text_input::State,
    input: String,
    data: Option<Work>,
    builder: WorkBuilder,
    input_message: String,
    timeline: Timeline,
}

impl BookSingleUI {
    fn parse_input(&mut self, text: &str) {
        let (text, msg) = if let Some((text, msg)) = text.split_once('#') {
            (text, msg)
        } else {
            (text, "")
        };

        let now = self.timeline.time_now();

        let orig = std::mem::take(&mut self.builder.task);

        let entries: Vec<_> = text.splitn(4, ' ').collect();
        self.builder.start = ParseResult::None;
        self.builder.end = ParseResult::None;
        self.builder.msg = if msg.is_empty() {
            Some(msg.to_string())
        } else {
            None
        };

        match *entries.as_slice() {
            [s, e, i, m0, ref m @ ..] => {
                if msg.is_empty() {
                    self.builder.start = parse_input_rel(now, s, true);
                    self.builder.end = parse_input(now, e);
                    self.builder.task = parse_issue(i);
                    self.builder.msg = Some(if m.is_empty() {
                        m0.to_string()
                    } else {
                        format!("{} {}", m0, m.join(" "))
                    });
                } else {
                    self.builder.task = ParseResult::Invalid(())
                }
            }
            [s, e, i] => {
                self.builder.start = parse_input_rel(now, s, true);
                self.builder.end = parse_input(now, e);
                self.builder.task = parse_issue(i);
            }
            [s, i] => {
                self.builder.start = parse_input_rel(now, s, true);
                self.builder.task = parse_issue(i);
            }
            [i] => {
                self.builder.task = parse_issue(i);
            }
            [] => {
                self.builder.start = ParseResult::Incomplete;
            }
        };

        if self.builder.needs_clipboard() && matches!(orig, ParseResult::Valid(_)) {
            self.builder.task = orig;
            self.builder.clipboard_reading = "";
        }
    }

    pub fn for_active_day(settings: &Settings, active_day: Option<&ActiveDay>) -> Box<Self> {
        let actions = active_day.map(|a| a.actions()).unwrap_or_default();
        Box::new(Self {
            input_state: text_input::State::focused(),
            input: "".to_string(),
            data: None,
            builder: Default::default(),
            input_message: input_message("Book issue", actions),
            timeline: settings.timeline.clone(),
        })
    }

    fn follow_up_msg(&mut self) -> Option<Message> {
        if self.builder.needs_clipboard() {
            self.builder.clipboard_reading = "reading...";
            Some(Message::ReadClipboard)
        } else {
            None
        }
    }

    fn on_submit_message(&self, settings: &Settings) -> Message {
        self.builder
            .try_build(settings.timeline.time_now(), settings)
            .map(|e| Message::StoreAction(crate::data::Action::Work(e)))
            .unwrap_or_default()
    }
}

impl MainView for BookSingleUI {
    fn new(_settings: &Settings) -> Box<Self> {
        Box::new(Self {
            input_state: text_input::State::focused(),
            builder: Default::default(),
            data: None,
            input: String::new(),
            input_message: input_message("Book issue", &[]),
            timeline: Arc::new(DefaultTimeline),
        })
    }

    fn view(&mut self, settings: &Settings) -> QElement {
        let on_submit = self.on_submit_message(settings);

        let msg = Text::new(&self.input_message);
        let input = TextInput::new(&mut self.input_state, "", &self.input, |s| {
            Message::Bs(BookSingleMessage::TextChanged(s))
        })
        .on_submit(on_submit);

        let now = settings.timeline.time_now();

        let status = Row::with_children(vec![
            text("Start:"),
            h_space(style::SPACE),
            time_info(now, self.builder.start.clone()),
            h_space(style::DSPACE),
            text("End:"),
            h_space(style::SPACE),
            time_info(now, self.builder.end.clone()),
            h_space(style::DSPACE),
            text("Task:"),
            h_space(style::SPACE),
            task_info(self.builder.task.as_ref(), self.builder.clipboard_reading),
            h_space(style::DSPACE),
            text("Message:"),
            h_space(style::SPACE),
            text(
                self.builder
                    .msg
                    .as_ref()
                    .map(|e| e.as_ref())
                    .unwrap_or("<no message>"),
            ),
        ]);

        Column::with_children(vec![
            msg.into(),
            v_space(style::SPACE),
            input.into(),
            v_space(style::SPACE),
            status.into(),
        ])
        .into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Bs(BookSingleMessage::TextChanged(msg)) => {
                self.parse_input(&msg);
                self.input = msg;
                self.follow_up_msg()
            }
            Message::ClipboardValue(v) => {
                self.builder.apply_clipboard(v);
                None
            }
            Message::StoreSuccess => Some(Message::Exit),
            _ => self.follow_up_msg(),
        }
    }
}

fn text<'a>(t: &str) -> QElement<'a> {
    Text::new(t.to_string()).into()
}

fn time_info<'a>(now: Time, v: ParseResult<Time, ()>) -> QElement<'a> {
    Text::new(
        v.get_with_default(now)
            .map(|e| e.to_string())
            .unwrap_or_else(|| "invalid".to_string()),
    )
    .into()
}

fn task_info<'a>(v: ParseResult<&'a Task, &'a ()>, clipboard: &'a str) -> QElement<'a> {
    match v {
        ParseResult::Valid(t) => task_text(t),
        ParseResult::Invalid(_) => text("invalid"),
        ParseResult::Incomplete => text("incomplete"),
        ParseResult::None => text(clipboard),
    }
}

fn task_text(t: &Task) -> QElement {
    match t {
        Task::Jira(i) => text(&i.ident),
        Task::Admin => text("Admin"),
        Task::Meeting => text("Meeting"),
    }
}

fn parse_issue_clipboard(input: &str) -> Option<JiraIssue> {
    let pattern = ISSUE_NUM.deref();
    let c = pattern.captures(input)?;
    let id = c.name("id")?;

    Some(JiraIssue {
        ident: id.as_str().to_string(),
        description: c.name("comment").map(|m| m.as_str().to_string()),
        default_action: None,
    })
}

fn parse_issue(input: &str) -> ParseResult<Task, ()> {
    if input == "m" {
        ParseResult::Valid(Task::Meeting)
    } else if input == "a" {
        ParseResult::Valid(Task::Admin)
    } else if input == "c" {
        ParseResult::None
    } else if let Some((key, num)) = input.split_once('-') {
        if key.chars().all(|ch| ch.is_ascii_alphabetic())
            && num.chars().all(|ch| ch.is_ascii_digit())
            && !key.is_empty()
            && !num.is_empty()
        {
            ParseResult::Valid(Task::Jira(JiraIssue {
                ident: format!("{}-{}", key, num),
                description: None,
                default_action: None,
            }))
        } else {
            ParseResult::Invalid(())
        }
    } else {
        ParseResult::Invalid(())
    }
}

#[derive(Default, Debug)]
struct WorkBuilder {
    start: ParseResult<Time, ()>,
    end: ParseResult<Time, ()>,
    task: ParseResult<Task, ()>,
    msg: Option<String>,
    clipboard_reading: &'static str,
}

impl WorkBuilder {
    fn needs_clipboard(&self) -> bool {
        matches!(self.task, ParseResult::None) && self.clipboard_reading.is_empty()
    }

    fn apply_clipboard(&mut self, value: Option<String>) {
        self.clipboard_reading = "";
        if let ParseResult::None = self.task {
            let value = value.as_deref().unwrap_or("");
            if !value.is_empty() {
                if let Some(ji) = parse_issue_clipboard(value) {
                    self.task = ParseResult::Valid(Task::Jira(ji));
                } else {
                    self.task = ParseResult::Invalid(());
                    self.clipboard_reading = "invalid clip"
                }
            } else {
                self.task = ParseResult::Invalid(());
                self.clipboard_reading = "no clipboard"
            }
        } else {
            eprintln!("Cannot apply clipboard");
            self.clipboard_reading = "unexpected";
        }
    }

    fn try_build(&self, now: Time, _settings: &Settings) -> Option<Work> {
        let start = self.start.get_with_default(now);

        let end = self.end.get_with_default(now);

        let task = self.task.clone().get();

        match (start, end, task) {
            (Some(start), Some(end), Some(task)) => {
                let description = if let Some(ref d) = self.msg {
                    d
                } else {
                    match task {
                        Task::Jira(JiraIssue {
                            default_action: Some(ref action),
                            ..
                        }) => action,
                        Task::Jira(JiraIssue {
                            description: Some(ref description),
                            ..
                        }) => description,
                        _ => return None,
                    }
                };

                let description = description.to_string();
                Some(Work {
                    start: start.into(),
                    end: end.into(),
                    task,
                    description,
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::data::{JiraIssue, Task, Work};
    use crate::parsing::parse_result::ParseResult;
    use crate::parsing::time::Time;
    use crate::ui::book_single::{parse_issue_clipboard, BookSingleMessage, BookSingleUI};
    use crate::ui::{MainView, Message};
    use crate::Settings;

    #[test]
    fn test_parse_valid_clipboard() {
        assert_eq!(
            parse_issue_clipboard("CLIP-12345"),
            Some(JiraIssue {
                ident: "CLIP-12345".to_string(),
                description: None,
                default_action: None,
            })
        );
    }

    #[test]
    fn book_single_integration_test() {
        let settings = Settings::default();
        let mut bs = BookSingleUI::for_active_day(&settings, None);
        let text_changed_msg = bs.update(Message::Bs(BookSingleMessage::TextChanged(
            "1 10 c comment".to_string(),
        )));

        assert!(matches!(text_changed_msg, Some(Message::ReadClipboard)));

        assert_eq!(bs.builder.clipboard_reading, "reading...");
        assert_eq!(bs.builder.task, ParseResult::None);
        assert_eq!(bs.builder.start, ParseResult::Valid(Time::hm(1, 0)));
        assert_eq!(bs.builder.end, ParseResult::Valid(Time::hm(10, 0)));
        assert_eq!(bs.builder.msg.as_deref(), Some("comment"));

        let clip_value = bs.update(Message::ClipboardValue(Some("CLIP-1234".to_string())));
        assert!(clip_value.is_none());

        assert_eq!(bs.builder.clipboard_reading, "");
        assert_eq!(
            bs.builder.task,
            ParseResult::Valid(Task::Jira(JiraIssue {
                ident: "CLIP-1234".to_string(),
                description: None,
                default_action: None,
            }))
        );

        let work = bs.builder.try_build(Time::hm(11, 0), &settings).unwrap();
        assert_eq!(
            work,
            Work {
                start: Time::hm(1, 0).into(),
                end: Time::hm(10, 0).into(),
                task: Task::Jira(JiraIssue::create("CLIP-1234").unwrap()),
                description: "comment".to_string()
            }
        );

        let next_letter = bs.update(Message::Bs(BookSingleMessage::TextChanged(
            "1 10 c comment1".to_string(),
        )));

        assert_eq!(bs.builder.clipboard_reading, "");
        assert!(matches!(bs.builder.task, ParseResult::Valid(Task::Jira(_))));
        assert_eq!(bs.builder.start, ParseResult::Valid(Time::hm(1, 0)));
        assert_eq!(bs.builder.end, ParseResult::Valid(Time::hm(10, 0)));
        assert_eq!(bs.builder.msg.as_deref(), Some("comment1"));

        assert!(matches!(next_letter, None));
        let on_submit = bs.on_submit_message(&settings);
        assert!(
            matches!(
                on_submit,
                Message::StoreAction(crate::data::Action::Work(_))
            ),
            "{:?}",
            on_submit
        );
    }
}
