use crate::data::JiraIssue;
use crate::data::Work;
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::time_relative::TimeRelative;
use crate::parsing::{IssueParsed, IssueParser};
use crate::Settings;
use lazy_static::lazy_static;

lazy_static! {
    static ref ISSUE_NUM: regex::Regex =
        regex::RegexBuilder::new(r"(?P<id>(?:[a-zA-Z]+)-(?:[0-9]+))(?:(?:\W)+(?P<comment>.*))?")
            .build()
            .unwrap();
    static ref SEPARATOR: regex::Regex = regex::RegexBuilder::new(r"[ \t\n\r]+").build().unwrap();
}

#[derive(Debug, Eq, PartialEq)]
pub enum ClipRead {
    None,
    Reading,
    DoRead,
    Invalid,
    NoClip,
    Unexpected,
}

impl Default for ClipRead {
    fn default() -> Self {
        ClipRead::None
    }
}

impl ClipRead {
    pub fn as_str(&self) -> &'static str {
        match self {
            ClipRead::None => "",
            ClipRead::Reading => "reading...",
            ClipRead::DoRead => "read clipboard",
            ClipRead::Invalid => "invalid clipboard",
            ClipRead::NoClip => "no clipboard",
            ClipRead::Unexpected => "unexpected",
        }
    }
}

#[derive(Default, Debug)]
pub(super) struct WorkBuilder {
    pub start: ParseResult<Time, ()>,
    pub end: ParseResult<Time, ()>,
    pub task: ParseResult<JiraIssue, ()>,
    pub msg: Option<String>,
    pub clipboard_reading: ClipRead,
    pub last_task_input: String,
}

impl WorkBuilder {
    pub(super) fn needs_clipboard(&self) -> bool {
        matches!(self.clipboard_reading, ClipRead::DoRead)
    }

    pub(super) fn parse_input(&mut self, settings: &Settings, text: &str) {
        parse(self, &settings, text)
    }

    /*pub(super) fn parse_input(&mut self, text: &str, now: Time) {
            let (text, msg) = if let Some((text, msg)) = text.split_once('#') {
                (text, msg)
            } else {
                (text, "")
            };

            let orig = std::mem::take(&mut self.task);

            let entries: Vec<_> = SEPARATOR.splitn(text, 4).collect();

            self.start = ParseResult::None;
            self.end = ParseResult::None;
            self.msg = if msg.is_empty() {
                Some(msg.to_string())
            } else {
                None
            };

            match *entries.as_slice() {
                [s, e, i, m0, ref m @ ..] => {
                    if msg.is_empty() {
                        self.start = parse_input_rel(now, s, true);
                        self.end = parse_input(now, e);
                        self.task = parse_issue(i);
                        self.msg = Some(if m.is_empty() {
                            m0.to_string()
                        } else {
                            format!("{} {}", m0, m.join(" "))
                        });
                    } else {
                        self.task = ParseResult::Invalid(())
                    }
                }
                [s, e, i] => {
                    self.start = parse_input_rel(now, s, true);
                    self.end = parse_input(now, e);
                    self.task = parse_issue(i);
                }
                [s, i] => {
                    self.start = parse_input_rel(now, s, true);
                    self.task = parse_issue(i);
                }
                [i] => {
                    self.task = parse_issue(i);
                }
                [] => {
                    self.start = ParseResult::Incomplete;
                }
            };

            if self.needs_clipboard() && matches!(orig, ParseResult::Valid(_)) {
                self.task = orig;
                self.clipboard_reading = "";
            }
        }
    */
    pub(super) fn apply_clipboard(&mut self, value: Option<String>) {
        self.clipboard_reading = ClipRead::None;
        if let ParseResult::None = self.task {
            let value = value.as_deref().unwrap_or("");
            if !value.is_empty() {
                if let Some(ji) = parse_issue_clipboard(value) {
                    self.task = ParseResult::Valid(ji);
                } else {
                    self.task = ParseResult::Invalid(());
                    self.clipboard_reading = ClipRead::Invalid;
                }
            } else {
                self.task = ParseResult::Invalid(());
                self.clipboard_reading = ClipRead::NoClip
            }
        } else {
            eprintln!("Cannot apply clipboard");
            self.clipboard_reading = ClipRead::Unexpected;
        }
    }

    pub(super) fn try_build(&self, now: Time, _settings: &Settings) -> Option<Work> {
        let start = self.start.get_with_default(now);

        let end = self.end.get_with_default(now);

        let task = self.task.clone().get();

        match (start, end, task) {
            (Some(start), Some(end), Some(task)) => {
                let description = if let Some(ref d) = self.msg {
                    d
                } else {
                    match task {
                        JiraIssue {
                            default_action: Some(ref action),
                            ..
                        } => action,
                        JiraIssue {
                            description: Some(ref description),
                            ..
                        } => description,
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

pub(super) fn parse_issue_clipboard(input: &str) -> Option<JiraIssue> {
    let c = ISSUE_NUM.captures(input)?;
    let id = c.name("id")?;

    Some(JiraIssue {
        ident: id.as_str().to_string(),
        description: c.name("comment").map(|m| m.as_str().to_string()),
        default_action: None,
    })
}

fn parse(b: &mut WorkBuilder, settings: &Settings, input: &str) {
    enum TorD {
        Time(Time),
        Dur(TimeRelative),
    }

    fn parse_time(now: Time, input: &str) -> (ParseResult<TorD, ()>, &str) {
        let t1 = Time::parse_prefix(input);
        let t1 = match t1 {
            (ParseResult::None | ParseResult::Incomplete, _) => {
                let (tr, rest) = TimeRelative::parse_relative(input);
                (
                    tr.and_then(|r| now.try_add_relative(r).into())
                        .map(TorD::Time),
                    rest,
                )
            }
            (absolute, rest) => (absolute.map(TorD::Time), rest),
        };

        match t1 {
            (ParseResult::None | ParseResult::Incomplete, _) => {
                let (rel, rest) = TimeRelative::parse_duration(input);
                (rel.map(TorD::Dur), rest)
            }
            time => time,
        }
    }

    let now = settings.timeline.time_now();
    let input = input.trim_start();

    let (t1, rest) = parse_time(now, input);
    let rest = rest.trim_start();
    // just avoid double_parsing when input contains no times at all
    // if may be removed for better readability but worse performance
    let (t2, rest) = if t1.is_empty() {
        (ParseResult::None, rest)
    } else {
        parse_time(now, rest)
    };

    let (start, end) = match (t1, t2) {
        (ParseResult::Valid(TorD::Dur(_)), ParseResult::Valid(TorD::Dur(_))) => {
            (ParseResult::Invalid(()), ParseResult::Invalid(()))
        }
        (ParseResult::Valid(TorD::Time(s)), ParseResult::Valid(TorD::Time(e))) => {
            (ParseResult::Valid(s), ParseResult::Valid(e))
        }
        (ParseResult::Valid(TorD::Time(s)), ParseResult::Valid(TorD::Dur(dur))) => {
            (ParseResult::Valid(s), s.try_add_relative(dur).into())
        }
        (ParseResult::Valid(TorD::Dur(dur)), ParseResult::Valid(TorD::Time(e))) => {
            let s: ParseResult<Time, ()> = e.try_add_relative(-dur).into();
            (s, ParseResult::Valid(e))
        }
        (ParseResult::Valid(TorD::Dur(dur)), ParseResult::None | ParseResult::Incomplete) => {
            let s: ParseResult<Time, ()> = now.try_add_relative(-dur).into();
            (s, ParseResult::Valid(now))
        }
        _ => (ParseResult::Invalid(()), ParseResult::Invalid(())),
    };

    let (
        IssueParsed {
            r: issue, input, ..
        },
        comment,
    ) = parse_from_issue(&settings.issue_parser, rest.trim_start());

    let old_issue = std::mem::take(&mut b.task);

    b.start = start;
    b.end = end;
    b.task = issue;
    b.msg = comment.map(|s| s.to_owned());

    if matches!(b.task, ParseResult::None) {
        if input != b.last_task_input.as_str() {
            b.clipboard_reading = ClipRead::DoRead;
        } else {
            b.task = old_issue;
        }
    } else {
        b.clipboard_reading = ClipRead::None;
    }
    b.last_task_input = input.to_string();
}

fn parse_from_issue<'a, 'b>(
    ip: &'b IssueParser,
    input: &'a str,
) -> (IssueParsed<'a>, Option<&'a str>) {
    let issue = ip.parse_task(input);
    match issue {
        i
        @
        IssueParsed {
            r: ParseResult::Invalid(_) | ParseResult::Incomplete,
            ..
        } => return (i, None),
        _ => (),
    };
    let rest = issue.rest.trim();
    let comment = if rest.is_empty() { None } else { Some(rest) };
    (issue, comment)
}
