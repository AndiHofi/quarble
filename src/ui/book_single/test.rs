#![cfg(test)]

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::conf::{into_settings_ref, SettingsRef};
use crate::data::test_support::time;
use crate::data::{ActiveDayBuilder, JiraIssue, Location, RecentIssuesRef, Work};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::{parse_issue_clipboard, JiraIssueParser};
use crate::ui::book_single::{BookSingleMessage, BookSingleUI};
use crate::ui::clip_read::ClipRead;
use crate::ui::single_edit_ui::SingleEditUi;
use crate::ui::stay_active::StayActive;
use crate::ui::{MainView, Message};
use crate::util::StaticTimeline;
use crate::Settings;

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
    let settings = Settings {
        timeline: Arc::new(tl),
        issue_parser: JiraIssueParser::new(BTreeMap::from_iter([('a', meeting())].into_iter())),
        ..Default::default()
    };

    let settings = into_settings_ref(settings);
    let active_day = ActiveDayBuilder {
        day: settings.load().timeline.today(),
        active_issue: None,
        main_location: Location::Office,
        actions: Vec::new(),
    }
    .build();

    BookSingleUI::for_active_day(
        settings.clone(),
        RecentIssuesRef::empty(settings),
        Some(&active_day),
    )
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
    let settings = into_settings_ref(Settings::default());
    let mut bs =
        BookSingleUI::for_active_day(settings.clone(), RecentIssuesRef::empty(settings), None);
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

    let work = bs.builder.try_build(Time::hm(11, 0)).unwrap();
    assert_eq!(
        work,
        Work {
            start: Time::hm(1, 0),
            end: Time::hm(10, 0),
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
    let on_submit = bs.update(Message::SubmitCurrent(StayActive::Yes));
    assert!(
        matches!(
            on_submit,
            Some(Message::StoreAction(
                StayActive::Yes,
                crate::data::Action::Work(_),
            ))
        ),
        "{:?}",
        on_submit
    );
}

#[test]
fn applies_recent_issues() {
    let (settings, _, mut ui) = setup_test_ui();

    ui.parse_input("10 11 r1");

    let result = ui.builder.try_build(settings.load().timeline.time_now());
    assert_eq!(
        result,
        Some(Work {
            start: time("10"),
            end: time("11"),
            task: JiraIssue {
                ident: "RECENT-1".to_string(),
                description: Some("Description".to_string()),
                default_action: Some("Default action".to_string())
            },
            description: "Default action".to_string()
        })
    )
}

#[test]
fn can_adapt_recent_issue() {
    let (settings, _, mut ui) = setup_test_ui();
    ui.parse_input("10 11 r1 modified action");

    let result = ui.builder.try_build(settings.load().timeline.time_now());
    assert_eq!(
        result,
        Some(Work {
            start: time("10"),
            end: time("11"),
            task: JiraIssue {
                ident: "RECENT-1".to_string(),
                description: Some("Description".to_string()),
                default_action: Some("Default action".to_string())
            },
            description: "modified action".to_string()
        })
    )
}

fn setup_test_ui() -> (SettingsRef, RecentIssuesRef, Box<BookSingleUI>) {
    let settings = into_settings_ref(Settings {
        max_recent_issues: 10,
        timeline: StaticTimeline::parse("2022-1-15 12:00").into(),
        ..Default::default()
    });

    let recent = RecentIssuesRef::empty(settings.clone());
    recent.issue_used_with_comment(
        &JiraIssue {
            ident: "RECENT-1".to_string(),
            default_action: Some("Default action".to_string()),
            description: Some("Description".to_string()),
        },
        None,
    );

    let ui = BookSingleUI::for_active_day(
        settings.clone(),
        recent.clone(),
        Some(
            &ActiveDayBuilder {
                day: settings.load().timeline.today(),
                main_location: Location::Office,
                active_issue: None,
                actions: vec![],
            }
            .build(),
        ),
    );

    (settings, recent, ui)
}
