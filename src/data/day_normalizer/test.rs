use super::*;
use crate::data::day_normalizer::day_splits;
use crate::data::{JiraIssue, Location, WorkEnd};
use crate::parsing;
use crate::parsing::time::Time;
use crate::parsing::time_limit::{TimeLimit, TimeRange};
use crate::ui::fast_day_start::DayStartBuilder;
use crate::util::{DefaultTimeline, TimelineProvider};
use std::collections::BTreeSet;
use std::sync::Arc;

#[test]
fn test_start_end_matching() {
    let actions = BTreeSet::from_iter([
        day_start("o9"),
        day_end("12"),
        day_start("13"),
        day_end("18"),
    ]);

    assert_eq!(
        start_end_spans(&actions),
        Ok(vec![
            TimeRange::new(Time::hm(9, 0), Time::hm(12, 0)),
            TimeRange::new(Time::hm(13, 0), Time::hm(18, 0))
        ])
    )
}

#[test]
fn test_no_entries() {
    let set = BTreeSet::new();
    assert_eq!(start_end_spans(&set), Ok(vec![]));
}

#[test]
fn test_missing_end() {
    let actions = BTreeSet::from_iter([day_start("h9")]);
    assert!(matches!(start_end_spans(&actions), Err(_)));

    let actions = BTreeSet::from_iter([day_start("h9"), day_end("10"), day_start("11")]);
    assert!(matches!(start_end_spans(&actions), Err(_)));
}

#[test]
fn test_missing_start() {
    let actions = BTreeSet::from_iter([day_end("9")]);
    assert!(matches!(start_end_spans(&actions), Err(_)));

    let actions = BTreeSet::from_iter([day_start("h9"), day_end("10"), day_end("11")]);
    assert!(matches!(start_end_spans(&actions), Err(_)));
}

#[test]
fn too_many_starts() {
    let actions = BTreeSet::from_iter([day_start("h9"), day_start("10"), day_end("11")]);
    assert_eq!(
        start_end_spans(&actions),
        Ok(vec![TimeRange::new(Time::hm(9, 0), Time::hm(11, 0))])
    );
}

#[test]
fn times_only_default_action() {
    let mut actions = BTreeSet::from_iter([day_start("h9"), day_end("12")]);
    let mut active_issue = Some(JiraIssue {
        ident: "D-15".to_string(),
        description: Some("Default issue".to_string()),
        default_action: Some("dev".to_string()),
    });

    let result = day_splits(&mut actions, &mut active_issue).unwrap();

    assert_eq!(
        result,
        vec![FilledRange {
            range: TimeRange::new(time("9"), time("12")),
            work: vec![We {
                id: "D-15".to_string(),
                description: "dev".to_string(),
                start: time("9"),
                end: time("12"),
                implicit: true
            },]
        }]
    )
}

#[test]
fn times_work_interrupting() {
    let mut actions = BTreeSet::from_iter([
        day_start("h9"),
        work("10", "10:10", "A-1", "review"),
        work("11", "11:21", "A-1", "review"),
        day_end("12"),
    ]);
    let mut active_issue = Some(JiraIssue {
        ident: "D-15".to_string(),
        description: Some("Default issue".to_string()),
        default_action: Some("dev".to_string()),
    });

    let result = day_splits(&mut actions, &mut active_issue).unwrap();

    assert_eq!(
        result,
        vec![FilledRange {
            range: TimeRange::new(time("9"), time("12")),
            work: vec![
                We {
                    id: "D-15".to_string(),
                    description: "dev".to_string(),
                    start: time("9"),
                    end: time("10"),
                    implicit: true
                },
                We {
                    id: "A-1".to_string(),
                    description: "review".to_string(),
                    start: time("10"),
                    end: time("10:10"),
                    implicit: false
                },
                We {
                    id: "D-15".to_string(),
                    description: "dev".to_string(),
                    start: time("10:10"),
                    end: time("11"),
                    implicit: true
                },
                We {
                    id: "A-1".to_string(),
                    description: "review".to_string(),
                    start: time("11"),
                    end: time("11:21"),
                    implicit: false
                },
                We {
                    id: "D-15".to_string(),
                    description: "dev".to_string(),
                    start: time("11:21"),
                    end: time("12"),
                    implicit: true
                },
            ]
        }]
    )
}

#[test]
fn combines_work() {
    let mut actions = BTreeSet::from_iter([
        day_start("h9"),
        work("10", "10:10", "A-1", "review"),
        work("11", "11:21", "A-1", "review"),
        day_end("12"),
    ]);
    let mut active_issue = Some(JiraIssue {
        ident: "D-15".to_string(),
        description: Some("Default issue".to_string()),
        default_action: Some("dev".to_string()),
    });

    let mut result = day_splits(&mut actions, &mut active_issue).unwrap();
    for e in result.iter_mut() {
        combine_bookings(&mut e.work);
    }

    assert_eq!(
        result,
        vec![FilledRange {
            range: TimeRange::new(time("9"), time("12")),
            work: vec![
                We {
                    id: "D-15".to_string(),
                    description: "dev".to_string(),
                    start: time("9"),
                    end: time("11:29"),
                    implicit: true
                },
                We {
                    id: "A-1".to_string(),
                    description: "review".to_string(),
                    start: time("11:29"),
                    end: time("12:00"),
                    implicit: false
                },
            ]
        }]
    )
}

#[test]
fn times_work_extending() {
    let mut actions = BTreeSet::from_iter([
        day_start("h9"),
        work("8", "10:10", "A-1", "review"),
        work("11", "12:21", "A-1", "review"),
        day_end("12"),
    ]);
    let mut active_issue = Some(JiraIssue {
        ident: "D-15".to_string(),
        description: Some("Default issue".to_string()),
        default_action: Some("dev".to_string()),
    });

    let result = day_splits(&mut actions, &mut active_issue).unwrap();

    assert_eq!(
        result,
        vec![FilledRange {
            range: TimeRange::new(time("8"), time("12:21")),
            work: vec![
                We {
                    id: "A-1".to_string(),
                    description: "review".to_string(),
                    start: time("8"),
                    end: time("10:10"),
                    implicit: false
                },
                We {
                    id: "D-15".to_string(),
                    description: "dev".to_string(),
                    start: time("10:10"),
                    end: time("11"),
                    implicit: true
                },
                We {
                    id: "A-1".to_string(),
                    description: "review".to_string(),
                    start: time("11"),
                    end: time("12:21"),
                    implicit: false
                },
            ]
        }]
    )
}

#[test]
fn test_round_bookings() {
    let mut actions = BTreeSet::from_iter([
        day_start("h855"),
        work("10", "10:10", "A-1", "review"),
        work("11", "11:21", "A-1", "review"),
        day_end("12"),
    ]);
    let mut active_issue = Some(JiraIssue {
        ident: "D-15".to_string(),
        description: Some("Default issue".to_string()),
        default_action: Some("dev".to_string()),
    });

    let mut result = day_splits(&mut actions, &mut active_issue).unwrap();
    for e in result.iter_mut() {
        round_bookings(e, NonZeroU32::new(15).unwrap()).unwrap();
    }
    assert_eq!(
        result,
        vec![FilledRange {
            range: TimeRange::new(time("9"), time("12")),
            work: vec![
                We {
                    id: "D-15".to_string(),
                    description: "dev".to_string(),
                    start: time("9"),
                    end: time("10"),
                    implicit: true
                },
                We {
                    id: "A-1".to_string(),
                    description: "review".to_string(),
                    start: time("10"),
                    end: time("10:15"),
                    implicit: false
                },
                We {
                    id: "D-15".to_string(),
                    description: "dev".to_string(),
                    start: time("10:15"),
                    end: time("11"),
                    implicit: true
                },
                We {
                    id: "A-1".to_string(),
                    description: "review".to_string(),
                    start: time("11"),
                    end: time("11:15"),
                    implicit: false
                },
                We {
                    id: "D-15".to_string(),
                    description: "dev".to_string(),
                    start: time("11:15"),
                    end: time("12"),
                    implicit: true
                },
            ]
        }]
    );
}

#[test]
fn test_round_bookings_minimizes_total_error() {
    let mut actions = BTreeSet::from_iter([
        day_start("h855"),
        work("10", "10:20", "A-1", "review"),
        work("11:10", "11:30", "A-1", "review"),
        day_end("12:05"),
    ]);
    let mut active_issue = Some(JiraIssue {
        ident: "D-15".to_string(),
        description: Some("Default issue".to_string()),
        default_action: Some("dev".to_string()),
    });

    let mut result = day_splits(&mut actions, &mut active_issue).unwrap();
    for e in result.iter_mut() {
        round_bookings(e, NonZeroU32::new(15).unwrap()).unwrap();
    }
    assert_eq!(
        result,
        vec![FilledRange {
            range: TimeRange::new(time("9"), time("12")),
            work: vec![
                We {
                    id: "D-15".to_string(),
                    description: "dev".to_string(),
                    start: time("9"),
                    end: time("10"),
                    implicit: true
                },
                We {
                    id: "A-1".to_string(),
                    description: "review".to_string(),
                    start: time("10"),
                    end: time("10:15"),
                    implicit: false
                },
                We {
                    id: "D-15".to_string(),
                    description: "dev".to_string(),
                    start: time("10:15"),
                    end: time("11"),
                    implicit: true
                },
                We {
                    id: "A-1".to_string(),
                    description: "review".to_string(),
                    start: time("11"),
                    end: time("11:15"),
                    implicit: false
                },
                We {
                    id: "D-15".to_string(),
                    description: "dev".to_string(),
                    start: time("11:15"),
                    end: time("12:00"),
                    implicit: true
                },
            ]
        }]
    );
}

lazy_static::lazy_static! {
static ref CONFIG: BreaksConfig = BreaksConfig {
        min_breaks_minutes: 45,
        min_work_time_minutes: 6 * 60,
        default_break: (time("12"), time("12:45")),
    };
}

#[test]
fn test_punch_breaks() {
    let mut entries = vec![We {
        id: "J-1".to_string(),
        description: "desc".to_string(),
        start: time("8"),
        end: time("17"),
        implicit: true,
    }];

    try_insert_break(&CONFIG, &mut entries);

    assert_eq!(
        &entries[..],
        &[
            We {
                id: "J-1".to_string(),
                description: "desc".to_string(),
                start: time("8"),
                end: time("12"),
                implicit: true
            },
            We {
                id: "J-1".to_string(),
                description: "desc".to_string(),
                start: time("12:45"),
                end: time("17"),
                implicit: true
            }
        ]
    )
}

#[test]
fn test_punches_no_break_manually_booked() {
    let mut entries = vec![We {
        id: "J-1".to_string(),
        description: "desc".to_string(),
        start: time("8"),
        end: time("17"),
        implicit: false,
    }];

    try_insert_break(&CONFIG, &mut entries);

    assert_eq!(
        &entries[..],
        &[We {
            id: "J-1".to_string(),
            description: "desc".to_string(),
            start: time("8"),
            end: time("17"),
            implicit: false
        },]
    )
}

#[test]
fn test_moves_breaks_forward_when_needed() {
    let mut entries = vec![
        We {
            id: "J-1".to_string(),
            description: "desc".to_string(),
            start: time("8"),
            end: time("11:00"),
            implicit: false,
        },
        We {
            id: "J-2".to_string(),
            description: "desc".to_string(),
            start: time("11:00"),
            end: time("12:00"),
            implicit: true,
        },
        We {
            id: "J-3".to_string(),
            description: "desc".to_string(),
            start: time("12:00"),
            end: time("17"),
            implicit: false,
        },
    ];

    try_insert_break(&CONFIG, &mut entries);

    assert_eq!(
        &entries[..],
        &[
            We {
                id: "J-1".to_string(),
                description: "desc".to_string(),
                start: time("8"),
                end: time("11:00"),
                implicit: false,
            },
            We {
                id: "J-2".to_string(),
                description: "desc".to_string(),
                start: time("11:00"),
                end: time("11:15"),
                implicit: true,
            },
            We {
                id: "J-3".to_string(),
                description: "desc".to_string(),
                start: time("12:00"),
                end: time("17"),
                implicit: false,
            },
        ]
    )
}

#[test]
fn test_places_breaks_correctly() {
    let mut entries = vec![
        We {
            id: "J-1".to_string(),
            description: "desc".to_string(),
            start: time("8"),
            end: time("11:00"),
            implicit: true,
        },
        We {
            id: "J-1".to_string(),
            description: "desc".to_string(),
            start: time("11"),
            end: time("11:45"),
            implicit: false,
        },
        We {
            id: "J-2".to_string(),
            description: "desc".to_string(),
            start: time("11:45"),
            end: time("12:30"),
            implicit: true,
        },
        We {
            id: "J-1".to_string(),
            description: "desc".to_string(),
            start: time("12:30"),
            end: time("14:00"),
            implicit: false,
        },
        We {
            id: "J-3".to_string(),
            description: "desc".to_string(),
            start: time("14:00"),
            end: time("17"),
            implicit: true,
        },
    ];

    try_insert_break(&CONFIG, &mut entries);

    assert_eq!(
        &entries[..],
        &[
            We {
                id: "J-1".to_string(),
                description: "desc".to_string(),
                start: time("8"),
                end: time("11:00"),
                implicit: true,
            },
            We {
                id: "J-1".to_string(),
                description: "desc".to_string(),
                start: time("11"),
                end: time("11:45"),
                implicit: false,
            },
            We {
                id: "J-1".to_string(),
                description: "desc".to_string(),
                start: time("12:30"),
                end: time("14:00"),
                implicit: false,
            },
            We {
                id: "J-3".to_string(),
                description: "desc".to_string(),
                start: time("14:00"),
                end: time("17"),
                implicit: true,
            },
        ]
    )
}

#[test]
fn test_moves_breaks_backwards_when_needed() {
    let mut entries = vec![
        We {
            id: "J-1".to_string(),
            description: "desc".to_string(),
            start: time("8"),
            end: time("12:45"),
            implicit: false,
        },
        We {
            id: "J-2".to_string(),
            description: "desc".to_string(),
            start: time("12:45"),
            end: time("17"),
            implicit: true,
        },
    ];

    try_insert_break(&CONFIG, &mut entries);

    assert_eq!(
        &entries[..],
        &[
            We {
                id: "J-1".to_string(),
                description: "desc".to_string(),
                start: time("8"),
                end: time("12:45"),
                implicit: false,
            },
            We {
                id: "J-2".to_string(),
                description: "desc".to_string(),
                start: time("13:30"),
                end: time("17"),
                implicit: true,
            },
        ]
    )
}

#[test]
fn integration_test() {
    let bookings = vec![
        day_start("h8"),
        issue_start("8:03", "A-1", "First", "doFirst"),
        work("8:00", "8:15", "M-1", "org"),
        work("8:30", "8:40", "M-1", "org"),
        issue_start("10:59", "A-2", "Second", "doSecond"),
        work("11:16", "11:29", "W-1", "meeting1"),
        work("12:31", "14:01", "W-2", "meeting2"),
        issue_start("13:38", "A-3", "Third", "doThird"),
        work("14", "1415", "M-1", "org"),
        day_end("1803"),
    ];

    let n = Normalizer {
        resolution: NonZeroU32::new(15).unwrap(),
        breaks_config: BreaksConfig {
            min_breaks_minutes: 45,
            min_work_time_minutes: 6 * 60,
            default_break: (time("1145"), time("1230")),
        },
        combine_bookings: true,
        add_break: true,
    };

    let normalized = n
        .create_normalized(&ActiveDay {
            active_issue: None,
            actions: BTreeSet::from_iter(bookings),
            day: Day::ymd(2022, 1, 6),
            main_location: Location::Home,
        })
        .unwrap();

    assert_eq!(
        normalized.orig_breaks,
        BreaksInfo {
            work_time: TimeRelative::from_minutes_sat(10 * 60 + 3),
            break_time: TimeRelative::ZERO,
            breaks: vec![]
        }
    );
    assert_eq!(
        normalized.final_breaks,
        BreaksInfo {
            work_time: TimeRelative::from_minutes_sat(10 * 60 - 45),
            break_time: TimeRelative::from_minutes_sat(45),
            breaks: vec![TimeRange::new(time("11:45"), time("12:30"))]
        }
    );

    assert_eq!(
        &normalized.entries[..],
        &[
            workn("8", "845", "M-1", "org"),
            workn("845", "1115", "A-1", "doFirst"),
            workn("1115", "11:45", "A-2", "doSecond"),
            workn("12:30", "12:45", "W-1", "meeting1"),
            workn("12:45", "14:15", "W-2", "meeting2"),
            workn("14:15", "18", "A-3", "doThird"),
        ]
    );
}

#[test]
fn integration_test_free_issue() {
    let bookings = vec![
        day_start("h8"),
        issue_start("8:03", "A-1", "First", "doFirst"),
        work("8:00", "8:15", "M-1", "org"),
        work("8:30", "8:40", "M-1", "org"),
        issue_start("10:59", "A-2", "Second", "doSecond"),
        work("11:16", "11:29", "W-1", "meeting1"),
        work("12:31", "14:01", "W-2", "meeting2"),
        day_end("1401"),
        work("16", "18", "M-1", "org"),
    ];

    let n = Normalizer {
        resolution: NonZeroU32::new(15).unwrap(),
        breaks_config: BreaksConfig {
            min_breaks_minutes: 45,
            min_work_time_minutes: 6 * 60,
            default_break: (time("1145"), time("1230")),
        },
        combine_bookings: true,
        add_break: true,
    };

    let normalized = n
        .create_normalized(&ActiveDay {
            active_issue: None,
            actions: BTreeSet::from_iter(bookings),
            day: Day::ymd(2022, 1, 6),
            main_location: Location::Home,
        })
        .unwrap();

    assert_eq!(
        normalized.orig_breaks,
        BreaksInfo {
            work_time: TimeRelative::from_minutes_sat(8 * 60 + 1),
            break_time: TimeRelative::from_minutes_sat(119),
            breaks: vec![TimeRange::new(time("14:01"), time("16"))]
        }
    );
    assert_eq!(
        normalized.final_breaks,
        BreaksInfo {
            work_time: TimeRelative::from_minutes_sat(8 * 60),
            break_time: TimeRelative::from_minutes_sat(120),
            breaks: vec![TimeRange::new(time("14"), time("16"))]
        }
    );

    assert_eq!(
        &normalized.entries[..],
        &[
            workn("8", "830", "M-1", "org"),
            workn("830", "11", "A-1", "doFirst"),
            workn("11", "12:15", "A-2", "doSecond"),
            workn("12:15", "12:30", "W-1", "meeting1"),
            workn("12:30", "14:00", "W-2", "meeting2"),
            workn("16", "18", "M-1", "org"),
        ]
    );
}

fn workn(start: &str, end: &str, issue: &str, description: &str) -> Work {
    Work {
        start: time(start),
        end: time(end),
        task: JiraIssue::create(issue).unwrap(),
        description: description.to_string(),
    }
}

fn work(start: &str, end: &str, issue: &str, description: &str) -> Action {
    Action::Work(workn(start, end, issue, description))
}

fn time(time: &str) -> Time {
    Time::parse_prefix(time).0.get().unwrap()
}

fn issue_start(start: &str, issue: &str, description: &str, action: &str) -> Action {
    Action::WorkStart(WorkStart {
        ts: time(start),
        task: JiraIssue {
            ident: issue.to_string(),
            description: if description.is_empty() {
                None
            } else {
                Some(description.to_string())
            },
            default_action: None,
        },
        description: action.to_string(),
    })
}

fn issue_end(end: &str, issue: &str) -> Action {
    Action::WorkEnd(WorkEnd {
        ts: time(end),
        task: JiraIssue::create(issue).unwrap(),
    })
}

fn day_start(input: &str) -> Action {
    let timeline: Arc<dyn TimelineProvider> = Arc::new(DefaultTimeline);
    let mut builder = DayStartBuilder::default();
    builder.parse_value(&timeline, &[TimeLimit::default()], input);
    builder.try_build(&timeline).map(Action::DayStart).unwrap()
}

fn day_end(input: &str) -> Action {
    let timeline = Arc::new(DefaultTimeline);
    parsing::parse_day_end(timeline.time_now(), input)
        .get()
        .map(|ts| Action::DayEnd(DayEnd { ts }))
        .unwrap()
}
