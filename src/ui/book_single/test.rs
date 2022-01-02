#![cfg(test)]

use crate::data::{JiraIssue, Work};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::{parse_issue_clipboard, IssueParser};
use crate::ui::book_single::{BookSingleMessage, BookSingleUI};
use crate::ui::clip_read::ClipRead;
use crate::ui::{MainView, Message};
use crate::util::StaticTimeline;
use crate::Settings;
use std::collections::BTreeMap;
use std::sync::Arc;

fn meeting() -> JiraIssue {
    JiraIssue {
        ident: "M-2".into(),
        description: Some("Meeting".into()),
        default_action: Some("daily".into()),
    }
}

fn make_ui(now: &str) -> Box<BookSingleUI> {
    let date_time = format!("2020-10-10 {}", now);
    let tl = StaticTimeline::parse(&date_time);
    let mut settings = Settings::default().with_timeline(tl);
    settings.issue_parser = IssueParser::new(BTreeMap::from_iter([('a', meeting())].into_iter()));
    BookSingleUI::for_active_day(Arc::new(settings), None)
}

#[test]
fn test_parse_input_absolute() {
    let mut ui = make_ui("12:00");
    ui.parse_input("9 915\ta  some comment");

    assert_eq!(ui.builder.start, ParseResult::Valid(Time::hm(9, 0)));
    assert_eq!(ui.builder.end, ParseResult::Valid(Time::hm(9, 15)));
    assert_eq!(ui.builder.task, ParseResult::Valid(meeting()));
    assert_eq!(ui.builder.msg.as_deref(), Some("some comment"));
}

#[test]
fn test_parse_input_duration_back() {
    let mut ui = make_ui("12:00");
    ui.parse_input("15m I-2 did stuff");
    assert_eq!(ui.builder.start, ParseResult::Valid(Time::hm(11, 45)));
    assert_eq!(ui.builder.end, ParseResult::Valid(Time::hm(12, 0)));
    assert_eq!(
        ui.builder.task,
        ParseResult::Valid(JiraIssue::create("I-2").unwrap())
    );
    assert_eq!(ui.builder.msg.as_deref(), Some("did stuff"));
}

#[test]
fn test_parse_input_duration_rel_now() {
    let mut ui = make_ui("12:00");
    ui.parse_input("30m n I-3 did stuff");
    assert_eq!(ui.builder.start, ParseResult::Valid(Time::hm(11, 30)));
    assert_eq!(ui.builder.end, ParseResult::Valid(Time::hm(12, 0)));
    assert_eq!(
        ui.builder.task,
        ParseResult::Valid(JiraIssue::create("I-3").unwrap())
    );
    assert_eq!(ui.builder.msg.as_deref(), Some("did stuff"));
}

#[test]
fn test_parse_input_duration_forward() {
    let mut ui = make_ui("12:00");
    ui.parse_input("n 15m I-4 did stuff");
    assert_eq!(ui.builder.start, ParseResult::Valid(Time::hm(12, 0)));
    assert_eq!(ui.builder.end, ParseResult::Valid(Time::hm(12, 15)));
    assert_eq!(
        ui.builder.task,
        ParseResult::Valid(JiraIssue::create("I-4").unwrap())
    );
    assert_eq!(ui.builder.msg.as_deref(), Some("did stuff"));
}

#[test]
fn test_parse_input_duration_absolute() {
    let mut ui = make_ui("12:00");
    ui.parse_input("9 +15m I-5 will finish soon");
    assert_eq!(ui.builder.start, ParseResult::Valid(Time::hm(9, 0)));
    assert_eq!(ui.builder.end, ParseResult::Valid(Time::hm(12, 15)));
    assert_eq!(
        ui.builder.task,
        ParseResult::Valid(JiraIssue::create("I-5").unwrap())
    );
    assert_eq!(ui.builder.msg.as_deref(), Some("will finish soon"));
}

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
    let settings = Arc::new(Settings::default());
    let mut bs = BookSingleUI::for_active_day(settings.clone(), None);
    let text_changed_msg = bs.update(Message::Bs(BookSingleMessage::TextChanged(
        "1 10 c comment".to_string(),
    )));

    assert!(
        matches!(text_changed_msg, Some(Message::ReadClipboard)),
        "{:?}",
        &text_changed_msg
    );

    assert_eq!(bs.builder.clipboard_reading, ClipRead::Reading);
    assert_eq!(bs.builder.task, ParseResult::None);
    assert_eq!(bs.builder.start, ParseResult::Valid(Time::hm(1, 0)));
    assert_eq!(bs.builder.end, ParseResult::Valid(Time::hm(10, 0)));
    assert_eq!(bs.builder.msg.as_deref(), Some("comment"));

    let clip_value = bs.update(Message::ClipboardValue(Some("CLIP-1234".to_string())));
    assert!(clip_value.is_none());

    assert_eq!(bs.builder.clipboard_reading, ClipRead::None);
    assert_eq!(
        bs.builder.task,
        ParseResult::Valid(JiraIssue {
            ident: "CLIP-1234".to_string(),
            description: None,
            default_action: None,
        })
    );

    let work = bs.builder.try_build(Time::hm(11, 0), &settings).unwrap();
    assert_eq!(
        work,
        Work {
            start: Time::hm(1, 0).into(),
            end: Time::hm(10, 0).into(),
            task: JiraIssue::create("CLIP-1234").unwrap(),
            description: "comment".to_string()
        }
    );

    let next_letter = bs.update(Message::Bs(BookSingleMessage::TextChanged(
        "1 10 c comment1".to_string(),
    )));

    assert_eq!(bs.builder.clipboard_reading, ClipRead::None);
    assert!(matches!(bs.builder.task, ParseResult::Valid(_)));
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
