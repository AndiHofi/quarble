use crate::data::{Action, CurrentWork, JiraIssue, Work};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::time_relative::TimeRelative;
use crate::parsing::{IssueParser, IssueParserWithRecent};
use crate::ui::clip_read::ClipRead;
use crate::ui::Message;
use crate::util::Timeline;
use anyhow::anyhow;
use lazy_static::lazy_static;
use regex::Regex;

/// UI model of the
#[derive(Default, Debug)]
pub struct WorkData {
    pub start: ParseResult<WTime, ()>,
    pub end: ParseResult<WTime, ()>,
    pub task: ParseResult<IssueInput, ()>,
    pub msg: Option<String>,
    pub description: Option<String>,
    pub clipboard_reading: ClipRead,
    pub last_task_input: String,
}

pub struct ValidWorkData<'a> {
    pub start: Time,
    pub end: Option<Time>,
    pub task: &'a str,
    pub msg: &'a str,
    pub description: Option<&'a str>,
}

impl WorkData {
    pub fn init(&mut self, value: Value) {
        match value {
            Value::Work(Work {
                start,
                end,
                task: JiraIssue {
                    ident, description, ..
                },
                description: action,
            }) => {
                self.start = ParseResult::Valid(WTime::Time(start));
                self.end = ParseResult::Valid(WTime::Time(end));
                self.task = ParseResult::Valid(IssueInput::Match(ident));
                self.msg = Some(action);
                self.description = description;
            }
            Value::CurrentWork(CurrentWork {
                start,
                task: JiraIssue {
                    ident, description, ..
                },
                description: action,
            }) => {
                self.start = ParseResult::Valid(WTime::Time(start));
                self.end = ParseResult::Valid(WTime::Empty);
                self.task = ParseResult::Valid(IssueInput::Match(ident));
                self.msg = Some(action);
                self.description = description;
            }
        }
    }

    pub fn try_as_work_data(&self, last: Option<Time>, now: Time) -> Option<ValidWorkData> {
        #[derive(Copy, Clone)]
        enum ABR {
            Absolute(Time),
            Relative(TimeRelative),
            None,
        }

        match self {
            WorkData {
                start: ParseResult::Valid(start),
                end: ParseResult::Valid(end),
                task: ParseResult::Valid(task),
                msg,
                description,
                clipboard_reading: ClipRead::None | ClipRead::NoClip,
                ..
            } => {
                let start_p = match start {
                    WTime::Time(t) => ABR::Absolute(*t),
                    WTime::Empty | WTime::Last => {
                        if let Some(last) = last {
                            ABR::Absolute(last)
                        } else {
                            ABR::None
                        }
                    }
                    WTime::Relative(relative) => ABR::Relative(*relative),
                    WTime::Now => ABR::Absolute(now),
                };

                let end_p = match end {
                    WTime::Time(t) => ABR::Absolute(*t),
                    WTime::Now => ABR::Absolute(now),
                    WTime::Last if last.is_some() => ABR::Absolute(last.unwrap()),
                    WTime::Relative(relative) => ABR::Relative(*relative),
                    WTime::Last | WTime::Empty => return None,
                };

                dbg!(task);

                let task = match task {
                    IssueInput::Match(task) => Some(task.as_str()),
                    IssueInput::Recent(task) => Some(task.ident.as_str()),
                    IssueInput::Clipboard => None,
                };

                dbg!(task);

                let start = match (start_p, end_p) {
                    (ABR::Absolute(start), _) => start,
                    (ABR::None, _) => return None,
                    (ABR::Relative(relative), ABR::Absolute(end)) => end + (-relative.abs()),
                    _ => return None,
                };

                let end = match (start_p, end_p) {
                    (_, ABR::Absolute(end)) => Some(end),
                    (_, ABR::None) => return None,
                    (ABR::Absolute(start), ABR::Relative(relative)) => Some(start + relative.abs()),
                    _ => return None,
                };

                if let (start, Some(task), Some(msg)) = (start, task, msg) {
                    Some(ValidWorkData {
                        start,
                        end,
                        task,
                        msg,
                        description: description.as_deref(),
                    })
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn needs_clipboard(&self) -> bool {
        matches!(self.task, ParseResult::Valid(IssueInput::Clipboard))
    }

    pub fn apply_clipboard(&mut self, clip_value: Option<String>) {
        dbg!(clip_value);
    }
}

fn empty_to_none<U, T: AsRef<[U]>>(possibly_empty: T) -> Option<T> {
    let slice: &[U] = possibly_empty.as_ref();
    if slice.is_empty() {
        None
    } else {
        Some(possibly_empty)
    }
}

#[derive(Debug, Copy, Clone)]
pub enum WTime {
    Last,
    Now,
    Time(Time),
    Relative(TimeRelative),
    Empty,
}

impl std::fmt::Display for WTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WTime::Last => f.write_str("last"),
            WTime::Now => f.write_str("now"),
            WTime::Empty => f.write_str(""),
            WTime::Relative(r) => std::fmt::Display::fmt(r, f),
            WTime::Time(t) => std::fmt::Display::fmt(t, f),
        }
    }
}

#[derive(Debug)]
pub enum IssueInput {
    Recent(JiraIssue),
    Match(String),
    Clipboard,
}

#[derive(Debug)]
pub enum Value {
    Work(Work),
    CurrentWork(CurrentWork),
}

impl TryFrom<Action> for Value {
    type Error = anyhow::Error;
    fn try_from(a: Action) -> Result<Self, Self::Error> {
        match a {
            Action::Work(w) => Ok(Value::Work(w)),
            Action::CurrentWork(c) => Ok(Value::CurrentWork(c)),
            e => Err(anyhow!("Neither work, or current work: {e:?}")),
        }
    }
}

impl From<Value> for Action {
    fn from(v: Value) -> Self {
        match v {
            Value::Work(w) => Action::Work(w),
            Value::CurrentWork(c) => Action::CurrentWork(c),
        }
    }
}

pub fn time_input(input: &str) -> (bool, Option<Message>) {
    if input.contains(' ') {
        (false, Some(Message::Next))
    } else {
        (TIME_INPUT.is_match(input), None)
    }
}

pub fn issue_input(input: &str) -> (bool, Option<Message>) {
    if input.contains(' ') {
        (false, Some(Message::Next))
    } else {
        (ISSUE_INPUT.is_match(input), None)
    }
}

pub fn comment_input(input: &str) -> (bool, Option<Message>) {
    if input.ends_with('#') {
        (false, Some(Message::Next))
    } else {
        (true, None)
    }
}

lazy_static! {
    static ref TIME_INPUT: Regex =
        Regex::new(r#"(^[\-+0-9:hm]*$)|(^(no?)w?$)|(^((la?)s?)t?$)"#).unwrap();
    static ref ISSUE_INPUT: Regex = Regex::new(r#"^[0-9:a-zA-Z\-]*$"#).unwrap();
}

pub fn parse_start(input: &str, timeline: &Timeline) -> ParseResult<WTime, ()> {
    let input = input.trim();
    match input {
        "" | "l" | "last" => ParseResult::Valid(WTime::Last),
        "-" => ParseResult::Valid(WTime::Empty),
        "n" | "now" => ParseResult::Valid(WTime::Now),
        input => parse_wtime(timeline, input),
    }
}

pub(crate) fn parse_end(input: &str, timeline: &Timeline) -> ParseResult<WTime, ()> {
    let input = input.trim();
    if input.is_empty() || input == "n" || input == "now" {
        ParseResult::Valid(WTime::Now)
    } else if input == "-" {
        ParseResult::Valid(WTime::Empty)
    } else {
        parse_wtime(timeline, input)
    }
}

fn parse_wtime(timeline: &Timeline, input: &str) -> ParseResult<WTime, ()> {
    let (result, rest) = TimeRelative::parse_duration(input);
    if let (ParseResult::Valid(result), "") = (result, rest) {
        return ParseResult::Valid(WTime::Relative(result));
    }

    let (result, rest) = Time::parse_with_offset(timeline, input);
    if !rest.is_empty() {
        ParseResult::Invalid(())
    } else {
        result.map(WTime::Time)
    }
}

pub(crate) fn parse_issue(
    input: &str,
    issue_parser: &IssueParserWithRecent,
) -> ParseResult<IssueInput, ()> {
    if input == "c" {
        ParseResult::Valid(IssueInput::Clipboard)
    } else if input.trim().is_empty() {
        ParseResult::None
    } else {
        issue_parser
            .parse_task(input)
            .r
            .map(|issue| IssueInput::Match(issue.ident))
    }
}
