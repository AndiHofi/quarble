use std::collections::BTreeMap;
use std::str::FromStr;

use lazy_static::lazy_static;
use regex::{Captures, Regex};

use crate::data::{JiraIssue, RecentIssues};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::rest;

lazy_static! {
    static ref ISSUE_SHORTCUT: Regex = Regex::new(r"^(?P<abbr>[a-zA-Z])\b").unwrap();
    static ref ISSUE: Regex = Regex::new(r"^(?P<id>([a-zA-Z]+-[0-9]+))").unwrap();
    static ref ISSUE_CLIPBOARD: Regex =
        Regex::new(r"(?P<id>(?:[a-zA-Z]+)-(?:[0-9]{0,3}))(?:(?:\W)+(?P<comment>.*))?").unwrap();
    static ref ISSUE_DESCRIPTION: Regex =
        Regex::new(r"^(?P<id>([a-zA-Z]+-[0-9]+))(?:\W+)(?P<comment>[^#]+)#").unwrap();
    static ref RECENT_ISSUE: Regex = Regex::new(r"^r(?P<recent>[1-9][0-9]*)").unwrap();
}

pub trait IssueParser {
    fn parse_task<'a>(&self, input: &'a str) -> IssueParsed<'a>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct IssueParsed<'a> {
    pub r: ParseResult<JiraIssue, ()>,
    pub input: &'a str,
    pub rest: &'a str,
}

#[derive(Clone, Debug, Default)]
pub struct JiraIssueParser {
    shortcuts: BTreeMap<char, JiraIssue>,
}

impl JiraIssueParser {
    pub fn new(shortcuts: BTreeMap<char, JiraIssue>) -> Self {
        Self { shortcuts }
    }

    pub fn shortcuts(&self) -> &BTreeMap<char, JiraIssue> {
        &self.shortcuts
    }
}

impl IssueParser for JiraIssueParser {
    fn parse_task<'a>(&self, input: &'a str) -> IssueParsed<'a> {
        if let Some(c) = ISSUE_DESCRIPTION.captures(input) {
            let id = c.name("id").unwrap().as_str();
            let comment = c.name("comment").unwrap().as_str();
            let comment = Some(comment.trim_end().to_string()).filter(|e| !e.is_empty());
            IssueParsed {
                r: ParseResult::Valid(JiraIssue {
                    ident: id.to_string(),
                    description: comment,
                    default_action: None,
                }),
                input: matching(&c),
                rest: rest(c, input),
            }
        } else if let Some(c) = ISSUE.captures(input) {
            let id = c.name("id").unwrap().as_str();
            IssueParsed {
                r: ParseResult::Valid(JiraIssue::create(id).unwrap()),
                input: matching(&c),
                rest: rest(c, input),
            }
        } else if let Some(c) = ISSUE_SHORTCUT.captures(input) {
            let abbr = c.name("abbr").unwrap().as_str();
            let ch: char = abbr.chars().next().unwrap();
            if ch == 'c' {
                IssueParsed {
                    r: ParseResult::None,
                    input: matching(&c),
                    rest: rest(c, input),
                }
            } else if let Some(i) = self.shortcuts.get(&ch) {
                IssueParsed {
                    r: ParseResult::Valid(i.clone()),
                    input: matching(&c),
                    rest: rest(c, input),
                }
            } else {
                IssueParsed {
                    r: ParseResult::Invalid(()),
                    input: matching(&c),
                    rest: rest(c, input),
                }
            }
        } else {
            IssueParsed {
                r: ParseResult::None,
                input: "",
                rest: input,
            }
        }
    }
}

pub struct IssueParserWithRecent<'a> {
    delegate: &'a JiraIssueParser,
    recent: &'a RecentIssues,
}

impl<'a> IssueParserWithRecent<'a> {
    pub fn new(delegate: &'a JiraIssueParser, recent: &'a RecentIssues) -> Self {
        Self { delegate, recent }
    }
}

impl<'a> IssueParser for IssueParserWithRecent<'a> {
    fn parse_task<'b>(&self, input: &'b str) -> IssueParsed<'b> {
        if let Some(c) = RECENT_ISSUE.captures(input) {
            let index = usize::from_str(c.name("recent").unwrap().as_str()).unwrap();
            let recent = self.recent.find_recent(index - 1).map(|r| r.issue.clone());
            IssueParsed {
                r: recent.ok_or(()).into(),
                input,
                rest: "",
            }
        } else {
            self.delegate.parse_task(input)
        }
    }
}

pub fn parse_issue_clipboard(input: &str) -> Option<JiraIssue> {
    let c = ISSUE_CLIPBOARD.captures(input)?;
    let id = c.name("id")?;

    Some(JiraIssue {
        ident: id.as_str().to_string(),
        description: c.name("comment").map(|m| m.as_str().to_string()),
        default_action: None,
    })
}

fn matching<'a, 'b>(c: &'b Captures<'a>) -> &'a str {
    c.get(0).unwrap().as_str()
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;

    use crate::data::JiraIssue;
    use crate::parsing::issue_parser::{IssueParsed, IssueParser, JiraIssueParser};
    use crate::parsing::parse_result::ParseResult;

    #[test]
    fn parse_shortcut() {
        let p = new_parser();

        assert_eq!(p.parse_task("a b"), valid_short("A-1", "a", " b"));
        assert_eq!(p.parse_task("b"), valid_short("B-1", "b", ""));
        assert_eq!(
            p.parse_task("ab"),
            IssueParsed {
                r: ParseResult::None,
                input: "",
                rest: "ab"
            }
        );
        assert_eq!(
            p.parse_task("x b"),
            IssueParsed {
                r: ParseResult::Invalid(()),
                input: "x",
                rest: " b"
            }
        );
    }

    #[test]
    fn parse_issue() {
        let p = new_parser();

        assert_eq!(p.parse_task("APM-452633\txy"), valid("APM-452633", "\txy"));
        assert_eq!(
            p.parse_task("APM--"),
            IssueParsed {
                r: ParseResult::None,
                input: "",
                rest: "APM--"
            }
        );
        assert_eq!(
            p.parse_task(" QU-1"),
            IssueParsed {
                r: ParseResult::None,
                input: "",
                rest: " QU-1"
            }
        );
        assert_eq!(
            p.parse_task("1QU-2"),
            IssueParsed {
                r: ParseResult::None,
                input: "",
                rest: "1QU-2"
            }
        );
        assert_eq!(
            p.parse_task("-2"),
            IssueParsed {
                r: ParseResult::None,
                input: "",
                rest: "-2"
            }
        );
        assert_eq!(p.parse_task("QU-98("), valid("QU-98", "("));

        assert_eq!(
            p.parse_task("QU-789 An issue#work"),
            valid_desc("QU-789 An issue#", "QU-789", "An issue", "work")
        );

        assert_eq!(
            p.parse_task("QU-789 \t An issue \t#work #1"),
            valid_desc("QU-789 \t An issue \t#", "QU-789", "An issue", "work #1")
        );

        assert_eq!(
            p.parse_task("QU-789 \t \t#work 1"),
            IssueParsed {
                r: ParseResult::Valid(JiraIssue::create("QU-789").unwrap()),
                input: "QU-789 \t \t#",
                rest: "work 1"
            }
        );
    }

    fn valid_short<'a>(id: &'a str, input: &'a str, rest: &'a str) -> IssueParsed<'a> {
        IssueParsed {
            r: ParseResult::Valid(JiraIssue::create(id).unwrap()),
            input,
            rest,
        }
    }

    fn valid_desc<'a>(
        input: &'a str,
        id: &'a str,
        desc: &'a str,
        rest: &'a str,
    ) -> IssueParsed<'a> {
        IssueParsed {
            r: ParseResult::Valid(JiraIssue {
                ident: id.to_string(),
                description: Some(desc.to_string()),
                default_action: None,
            }),
            input,
            rest,
        }
    }

    fn valid<'a>(id: &'a str, rest: &'a str) -> IssueParsed<'a> {
        IssueParsed {
            r: ParseResult::Valid(JiraIssue::create(id).unwrap()),
            input: id,
            rest,
        }
    }

    fn new_parser() -> JiraIssueParser {
        JiraIssueParser {
            shortcuts: BTreeMap::from_iter(
                [
                    ('a', JiraIssue::create("A-1").unwrap()),
                    ('b', JiraIssue::create("B-1").unwrap()),
                ]
                .into_iter(),
            ),
        }
    }
}
