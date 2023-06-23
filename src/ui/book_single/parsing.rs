use lazy_static::lazy_static;

use crate::data::Work;
use crate::data::{JiraIssue, RecentIssues};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::time_relative::TimeRelative;
use crate::parsing::{parse_issue_clipboard, IssueParsed, IssueParser, IssueParserWithRecent};
use crate::ui::clip_read::ClipRead;
use crate::util::Timeline;
use crate::Settings;
use regex::Regex;

lazy_static! {
    static ref SEPARATOR: Regex = Regex::new(r"[ \t\n\r]+").unwrap();
    static ref FROM_LAST: Regex = Regex::new(r"^l\b").unwrap();
}

pub enum StartTime {
    Last,
    Now,
    Time(Time),
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

    pub(super) fn parse_input(
        &mut self,
        settings: &Settings,
        recent_issues: &RecentIssues,
        last_end: Option<Time>,
        text: &str,
    ) {
        parse(self, settings, recent_issues, last_end, text)
    }

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

    pub(super) fn try_build(&self, now: Time) -> Option<Work> {
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
                    start,
                    end,
                    task,
                    description,
                })
            }
            _ => None,
        }
    }
}

pub(crate) enum TorD {
    Time(Time),
    Dur(TimeRelative),
    Last,
}

pub(crate) fn parse_time<'a, 'b>(
    timeline: &'b Timeline,
    input: &'a str,
) -> (ParseResult<TorD, ()>, &'a str) {
    let t1 = if let Some(c) = FROM_LAST.captures(input) {
        (ParseResult::Valid(TorD::Last), &input[c.len()..])
    } else {
        match Time::parse_with_offset(timeline, input) {
            (ParseResult::None | ParseResult::Incomplete, _) => {
                let (tr, rest) = TimeRelative::parse_relative(input);
                (
                    tr.and_then(|r| timeline.time_now().try_add_relative(r).into())
                        .map(TorD::Time),
                    rest,
                )
            }
            (absolute, rest) => (absolute.map(TorD::Time), rest),
        }
    };

    match t1 {
        (ParseResult::None | ParseResult::Incomplete, _) => {
            let (rel, rest) = TimeRelative::parse_duration(input);
            (rel.map(TorD::Dur), rest)
        }
        time => time,
    }
}

fn parse(
    b: &mut WorkBuilder,
    settings: &Settings,
    recent_issues: &RecentIssues,
    last_end: Option<Time>,
    input: &str,
) {

    let timeline = &settings.timeline;
    let input = input.trim_start();

    let (t1, rest) = parse_time(&settings.timeline, input);
    let rest = rest.trim_start();
    // just avoid double_parsing when input contains no times at all
    // if may be removed for better readability but worse performance
    let (t2, rest) = if t1.is_empty() {
        (ParseResult::None, rest)
    } else {
        parse_time(&settings.timeline, rest)
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
        (ParseResult::Valid(TorD::Last), ParseResult::Valid(TorD::Time(s)))
            if last_end.is_some() =>
        {
            (ParseResult::Valid(last_end.unwrap()), ParseResult::Valid(s))
        }
        (ParseResult::Valid(TorD::Last), ParseResult::Valid(TorD::Dur(s)))
            if last_end.is_some() =>
        {
            (
                ParseResult::Valid(last_end.unwrap()),
                ParseResult::Valid(last_end.unwrap() + s),
            )
        }
        (ParseResult::Valid(TorD::Dur(dur)), ParseResult::Valid(TorD::Time(e))) => {
            let s: ParseResult<Time, ()> = e.try_add_relative(-dur).into();
            (s, ParseResult::Valid(e))
        }
        (ParseResult::Valid(TorD::Dur(dur)), ParseResult::None | ParseResult::Incomplete) => {
            let now = timeline.time_now();
            let s: ParseResult<Time, ()> = now.try_add_relative(-dur).into();
            (s, ParseResult::Valid(now))
        }
        _ => (ParseResult::Invalid(()), ParseResult::Invalid(())),
    };

    let issue_parser = IssueParserWithRecent::new(&settings.issue_parser, recent_issues);

    let (
        IssueParsed {
            r: issue, input, ..
        },
        comment,
    ) = parse_from_issue(&issue_parser, rest.trim_start());

    let old_issue = std::mem::take(&mut b.task);

    b.start = start;
    b.end = end;
    b.msg = comment
        .or(issue
            .as_ref()
            .get()
            .and_then(|i| i.default_action.as_deref()))
        .map(|s| s.to_owned());
    b.task = issue;

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
    ip: &'b impl IssueParser,
    input: &'a str,
) -> (IssueParsed<'a>, Option<&'a str>) {
    let issue = ip.parse_task(input);
    if matches!(
        issue,
        IssueParsed {
            r: ParseResult::Invalid(_) | ParseResult::Incomplete,
            ..
        }
    ) {
        return (issue, None);
    }

    let rest = issue.rest.trim();
    let comment = if rest.is_empty() { None } else { Some(rest) };
    (issue, comment)
}
