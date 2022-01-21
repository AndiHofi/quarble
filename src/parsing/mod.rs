pub use input_parser::{parse_absolute, parse_day_end, parse_input_rel};
pub use issue_parser::{
    parse_issue_clipboard, IssueParsed, IssueParser, IssueParserWithRecent, JiraIssueParser,
};

mod input_parser;
mod issue_parser;
pub mod parse_result;
pub mod round_mode;
pub mod time;
pub mod time_limit;
pub mod time_relative;

fn rest<'a>(c: regex::Captures<'a>, input: &'a str) -> &'a str {
    &input[c.get(0).unwrap().end()..]
}
