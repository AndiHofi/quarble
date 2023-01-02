#![cfg(test)]

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::conf::{into_settings_ref, SettingsRef};
use crate::data::test_support::time;
use crate::data::{Action, ActiveDayBuilder, JiraIssue, Location, RecentIssuesRef, Work};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::{parse_issue_clipboard, JiraIssueParser};
use crate::ui::book_single::nparsing::{IssueInput, WTime};
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

fn make_ui_booked(now: &str, actions: Vec<Action>) -> Box<BookSingleUI> {
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
        actions,
    }
    .build();

    BookSingleUI::for_active_day(
        settings.clone(),
        RecentIssuesRef::empty(settings),
        Some(&active_day),
    )
}

fn make_ui(now: &str) -> Box<BookSingleUI> {
    make_ui_booked(now, Vec::new())
}

macro_rules! assert_not_m {
    ($actual:ident, $expected:pat) => {
        if matches!($actual, $expected) {
            panic!("Unexpected result: {:?}", $actual);
        }
    };

    ($actual:ident, $expected:pat, $msg:literal) => {
        if matches!($actual, $expected) {
            panic!("{}: {:?}", $msg, $actual);
        }
    };
}

macro_rules! assert_m {
    ($actual:expr, $expected:pat) => {{
        let response = $actual;
        if !matches!(response, $expected) {
            panic!("Unexpected result: {:?}", response);
        }
    }};

    ($actual:expr, $expected:pat, $msg:literal) => {{
        let response = $actual
        if !matches!(response, $expected) {
            panic!("{}: {:?}", $msg, response);
        }
    }};
}

struct LinearInput<'a> {
    start: &'a str,
    end: &'a str,
    issue: &'a str,
    comment: &'a str,
    description: &'a str,
}

impl Default for LinearInput<'static> {
    fn default() -> Self {
        LinearInput {
            start: "9",
            end: "10",
            issue: "ABC-123",
            comment: "Some Text",
            description: "",
        }
    }
}

fn without_char(input: &str, end: char) -> String {
    if input.ends_with(end) {
        input[0..input.len() - 1].to_string()
    } else {
        input.to_string()
    }
}

fn without_space(input: &str) -> String {
    without_char(input, ' ')
}

fn should_auto_next(input: &str, sep: char) -> bool {
    input.ends_with(sep)
}

fn assert_next_char(
    ui: &mut BookSingleUI,
    last_response: Option<Message>,
    input: &str,
    sep: char,
) -> Option<Message> {
    assert_not_m!(last_response, Some(Message::Next));
    if should_auto_next(input, sep) {
        let response = ui.update(Message::TextChanged(input.to_string()));
        assert_m!(response, Some(Message::Next))
    }

    ui.update(Message::Next)
}

fn assert_next(
    ui: &mut BookSingleUI,
    last_response: Option<Message>,
    input: &str,
) -> Option<Message> {
    assert_next_char(ui, last_response, input, ' ')
}

fn linear_input_test(ui: &mut BookSingleUI, input: LinearInput) {
    let description = input.comment.trim().into();
    let ident = input.issue.trim().to_string();
    linear_input_test_expected(
        ui,
        input,
        Work {
            start: Time::hm(9, 15),
            end: Time::hm(10, 15),
            task: JiraIssue::create(&ident).unwrap(),
            description,
        },
    )
}

fn linear_input_test_expected(ui: &mut BookSingleUI, input: LinearInput, expected: Work) {
    let response = ui.update(Message::TextChanged(without_space(input.start)));
    assert_next(ui, response, input.start);
    let response = ui.update(Message::TextChanged(without_space(input.end)));
    assert_next(ui, response, input.end);

    let response = ui.update(Message::TextChanged(without_space(input.issue)));
    assert_next(ui, response, input.issue);

    let response = ui.update(Message::TextChanged(without_char(input.comment, '#')));
    assert_next_char(ui, response, input.comment, '#');

    if !input.description.is_empty() {
        ui.update(Message::TextChanged(input.description.to_string()));
    }

    let response = ui.update(Message::SubmitCurrent(StayActive::Yes));
    assert_not_m!(response, Some(Message::Exit));
    assert_not_m!(response, None);
    assert_m!(
        &response,
        Some(Message::StoreAction(
            StayActive::Yes,
            Action::Work(Work { .. })
        ))
    );

    let Some(Message::StoreAction(_, Action::Work(work))) = response else {
        panic!("Expected work but got {:?}", response)
    };

    assert_eq!(work, expected)
}

#[test]
fn default_booking() {
    let mut ui = make_ui("9:00");
    linear_input_test_expected(
        &mut ui,
        LinearInput {
            start: "915 ",
            end: "1015 ",
            issue: "ABC-123 ",
            comment: "Some text ",
            description: "Description",
        },
        Work {
            start: time("915"),
            end: time("1015"),
            task: JiraIssue {
                ident: "ABC-123".to_string(),
                description: Some("Description".to_string()),
                default_action: None,
            },
            description: "Some text".to_string(),
        },
    );
}

#[test]
fn default_booking_from_now() {
    let mut ui = make_ui("9:15");
    linear_input_test(
        &mut ui,
        LinearInput {
            start: "now",
            end: "1015",
            ..Default::default()
        },
    );
}

#[test]
fn default_booking_from_default() {
    let mut ui = make_ui_booked(
        "11:00",
        vec![Action::Work(Work {
            start: time("9"),
            end: time("915"),
            task: JiraIssue::create("I-1").unwrap(),
            description: "d".to_string(),
        })],
    );

    // no start time input
    linear_input_test(
        &mut ui,
        LinearInput {
            start: "",
            end: "1015",
            issue: "I-2",
            comment: "some text",
            description: "",
        },
    );
}

#[test]
fn relative_end() {
    let mut ui = make_ui("8:00");

    linear_input_test_expected(
        &mut ui,
        LinearInput {
            start: "10",
            end: "1h30m",
            issue: "I-12",
            comment: "der text",
            description: "description",
        },
        Work {
            start: time("10"),
            end: time("1130"),
            task: JiraIssue {
                ident: "I-12".to_string(),
                description: Some("description".to_string()),
                default_action: None,
            },
            description: "der text".into(),
        },
    )
}

#[test]
fn relative_from_now() {
    let mut ui = make_ui("11:00");

    linear_input_test_expected(
        &mut ui,
        LinearInput {
            start: "-1h",
            end: "+1h",
            issue: "I-13",
            comment: "text",
            description: "",
        },
        Work {
            start: time("10"),
            end: time("12"),
            task: JiraIssue::create("I-13").unwrap(),
            description: "text".to_string(),
        },
    );
}
//  I-1231   Some stuff
#[test]
fn relative_from_now_description() {
    let mut ui = make_ui("11:00");

    linear_input_test_expected(
        &mut ui,
        LinearInput {
            start: "-1h",
            end: "+1h",
            issue: "I-13",
            comment: "text#",
            description: "My issue",
        },
        Work {
            start: time("10"),
            end: time("12"),
            task: JiraIssue {
                ident: "I-13".to_string(),
                description: Some("My issue".to_string()),
                default_action: None,
            },
            description: "text".to_string(),
        },
    );
}

// #[test]
// fn test_parse_input_absolute() {
//     let mut ui = make_ui("12:00");
//     ui.parse_input("9 915\ta  some comment");
//
//     assert_eq!(ui.builder.start, ParseResult::Valid(Time::hm(9, 0)));
//     assert_eq!(ui.builder.end, ParseResult::Valid(Time::hm(9, 15)));
//     assert_eq!(ui.builder.task, ParseResult::Valid(meeting()));
//     assert_eq!(ui.builder.msg.as_deref(), Some("some comment"));
// }
//
// #[test]
// fn test_parse_input_duration_back() {
//     let mut ui = make_ui("12:00");
//     ui.parse_input("15m I-2 did stuff");
//     assert_eq!(ui.builder.start, ParseResult::Valid(Time::hm(11, 45)));
//     assert_eq!(ui.builder.end, ParseResult::Valid(Time::hm(12, 0)));
//     assert_eq!(
//         ui.builder.task,
//         ParseResult::Valid(JiraIssue::create("I-2").unwrap())
//     );
//     assert_eq!(ui.builder.msg.as_deref(), Some("did stuff"));
// }
//
// #[test]
// fn test_parse_input_duration_rel_now() {
//     let mut ui = make_ui("12:00");
//     ui.parse_input("30m n I-3 did stuff");
//     assert_eq!(ui.builder.start, ParseResult::Valid(Time::hm(11, 30)));
//     assert_eq!(ui.builder.end, ParseResult::Valid(Time::hm(12, 0)));
//     assert_eq!(
//         ui.builder.task,
//         ParseResult::Valid(JiraIssue::create("I-3").unwrap())
//     );
//     assert_eq!(ui.builder.msg.as_deref(), Some("did stuff"));
// }
//
// #[test]
// fn test_parse_input_duration_forward() {
//     let mut ui = make_ui("12:00");
//     ui.parse_input("n 15m I-4 did stuff");
//     assert_eq!(ui.builder.start, ParseResult::Valid(Time::hm(12, 0)));
//     assert_eq!(ui.builder.end, ParseResult::Valid(Time::hm(12, 15)));
//     assert_eq!(
//         ui.builder.task,
//         ParseResult::Valid(JiraIssue::create("I-4").unwrap())
//     );
//     assert_eq!(ui.builder.msg.as_deref(), Some("did stuff"));
// }
//
// #[test]
// fn test_parse_input_duration_absolute() {
//     let mut ui = make_ui("12:00");
//     ui.parse_input("9 +15m I-5 will finish soon");
//     assert_eq!(ui.builder.start, ParseResult::Valid(Time::hm(9, 0)));
//     assert_eq!(ui.builder.end, ParseResult::Valid(Time::hm(12, 15)));
//     assert_eq!(
//         ui.builder.task,
//         ParseResult::Valid(JiraIssue::create("I-5").unwrap())
//     );
//     assert_eq!(ui.builder.msg.as_deref(), Some("will finish soon"));
// }
//
// #[test]
// fn test_parse_valid_clipboard() {
//     assert_eq!(
//         parse_issue_clipboard("CLIP-12345"),
//         Some(JiraIssue {
//             ident: "CLIP-12345".to_string(),
//             description: None,
//             default_action: None,
//         })
//     );
// }
//
// #[test]
// fn book_single_integration_test() {
//     let settings = into_settings_ref(Settings::default());
//     let mut bs =
//         BookSingleUI::for_active_day(settings.clone(), RecentIssuesRef::empty(settings), None);
//     let text_changed_msg = bs.update(Message::Bs(BookSingleMessage::TextChanged(
//         "1 10 c comment".to_string(),
//     )));
//
//     assert!(
//         matches!(text_changed_msg, Some(Message::ReadClipboard)),
//         "{:?}",
//         &text_changed_msg
//     );
//
//     assert_eq!(bs.builder.clipboard_reading, ClipRead::Reading);
//     assert_eq!(bs.builder.task, ParseResult::None);
//     assert_eq!(bs.builder.start, ParseResult::Valid(Time::hm(1, 0)));
//     assert_eq!(bs.builder.end, ParseResult::Valid(Time::hm(10, 0)));
//     assert_eq!(bs.builder.msg.as_deref(), Some("comment"));
//
//     let clip_value = bs.update(Message::ClipboardValue(Some("CLIP-1234".to_string())));
//     assert!(clip_value.is_none());
//
//     assert_eq!(bs.builder.clipboard_reading, ClipRead::None);
//     assert_eq!(
//         bs.builder.task,
//         ParseResult::Valid(JiraIssue {
//             ident: "CLIP-1234".to_string(),
//             description: None,
//             default_action: None,
//         })
//     );
//
//     let work = bs.builder.try_build(Time::hm(11, 0)).unwrap();
//     assert_eq!(
//         work,
//         Work {
//             start: Time::hm(1, 0),
//             end: Time::hm(10, 0),
//             task: JiraIssue::create("CLIP-1234").unwrap(),
//             description: "comment".to_string()
//         }
//     );
//
//     let next_letter = bs.update(Message::Bs(BookSingleMessage::TextChanged(
//         "1 10 c comment1".to_string(),
//     )));
//
//     assert_eq!(bs.builder.clipboard_reading, ClipRead::None);
//     assert!(matches!(bs.builder.task, ParseResult::Valid(_)));
//     assert_eq!(bs.builder.start, ParseResult::Valid(Time::hm(1, 0)));
//     assert_eq!(bs.builder.end, ParseResult::Valid(Time::hm(10, 0)));
//     assert_eq!(bs.builder.msg.as_deref(), Some("comment1"));
//
//     assert!(matches!(next_letter, None));
//     let on_submit = bs.update(Message::SubmitCurrent(StayActive::Yes));
//     assert!(
//         matches!(
//             on_submit,
//             Some(Message::StoreAction(
//                 StayActive::Yes,
//                 crate::data::Action::Work(_),
//             ))
//         ),
//         "{:?}",
//         on_submit
//     );
// }
//
// #[test]
// fn applies_recent_issues() {
//     let (settings, _, mut ui) = setup_test_ui();
//
//     ui.parse_input("10 11 r1");
//
//     let result = ui.builder.try_build(settings.load().timeline.time_now());
//     assert_eq!(
//         result,
//         Some(Work {
//             start: time("10"),
//             end: time("11"),
//             task: JiraIssue {
//                 ident: "RECENT-1".to_string(),
//                 description: Some("Description".to_string()),
//                 default_action: Some("Default action".to_string())
//             },
//             description: "Default action".to_string()
//         })
//     )
// }
//
// #[test]
// fn can_adapt_recent_issue() {
//     let (settings, _, mut ui) = setup_test_ui();
//     ui.parse_input("10 11 r1 modified action");
//
//     let result = ui.builder.try_build(settings.load().timeline.time_now());
//     assert_eq!(
//         result,
//         Some(Work {
//             start: time("10"),
//             end: time("11"),
//             task: JiraIssue {
//                 ident: "RECENT-1".to_string(),
//                 description: Some("Description".to_string()),
//                 default_action: Some("Default action".to_string())
//             },
//             description: "modified action".to_string()
//         })
//     )
// }

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
