use crate::data::JiraIssue;
use crate::parsing::parse_result::ParseResult;
use crate::parsing::rest;
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::BTreeMap;

lazy_static! {
    static ref ISSUE_SHORTCUT: Regex = Regex::new(r"^(?P<abbr>[a-zA-Z])\b").unwrap();
    static ref ISSUE: Regex = Regex::new(r"^(?P<id>([a-zA-Z]+-[0-9]+))").unwrap();
    static ref ISSUE_CLIPBOARD: regex::Regex =
        regex::RegexBuilder::new(r"(?P<id>(?:[a-zA-Z]+)-(?:[0-9]+))(?:(?:\W)+(?P<comment>.*))?")
            .build()
            .unwrap();
}

#[derive(Clone, Debug, Default)]
pub struct IssueParser {
    shortcuts: BTreeMap<char, JiraIssue>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct IssueParsed<'a> {
    pub r: ParseResult<JiraIssue, ()>,
    pub input: &'a str,
    pub rest: &'a str,
}

impl IssueParser {
    pub fn new(shortcuts: BTreeMap<char, JiraIssue>) -> Self {
        Self {
            shortcuts: shortcuts.clone(),
        }
    }

    pub fn shortcuts(&self) -> &BTreeMap<char, JiraIssue> {
        &self.shortcuts
    }

    pub fn parse_task<'a>(&self, input: &'a str) -> IssueParsed<'a> {
        if let Some(c) = ISSUE.captures(input) {
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
    use crate::data::JiraIssue;
    use crate::parsing::issue_parser::{IssueParsed, IssueParser};
    use crate::parsing::parse_result::ParseResult;
    use std::collections::BTreeMap;

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
    }

    fn valid_short<'a>(id: &'a str, input: &'a str, rest: &'a str) -> IssueParsed<'a> {
        IssueParsed {
            r: ParseResult::Valid(JiraIssue::create(id).unwrap()),
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

    fn new_parser() -> IssueParser {
        let p = IssueParser {
            shortcuts: BTreeMap::from_iter(
                [
                    ('a', JiraIssue::create("A-1").unwrap()),
                    ('b', JiraIssue::create("B-1").unwrap()),
                ]
                .into_iter(),
            ),
        };
        p
    }
}
