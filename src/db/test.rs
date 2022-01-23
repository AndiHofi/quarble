use crate::data::test_support::*;
use crate::data::*;

use crate::db::{DBResult, DB};
use crate::parsing::time::Time;
use crate::util::{DefaultTimeline, TimelineProvider};
use chrono::Datelike;
use std::ops::Deref;
use tempfile::TempDir;

lazy_static::lazy_static! {
   static ref DAY0: Day = Day::ymd(2022, 1, 10);
}

pub struct TmpDB(DB, TempDir);

impl TmpDB {
    pub fn new() -> Self {
        let db_dir = TempDir::new().unwrap();
        let db = DB::init(db_dir.as_ref()).unwrap();
        TmpDB(db, db_dir)
    }
}

impl Deref for TmpDB {
    type Target = DB;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[test]
fn get_day_does_not_store() {
    let db = TmpDB::new();
    let _ = db.get_day(*DAY0).unwrap();
    assert_eq!(db.load_day(*DAY0).unwrap(), None);
}

#[test]
fn test_load_just_stored_day() {
    let db = TmpDB::new();

    let mut day0_data = ActiveDay::new(*DAY0, Location::Office, None);
    day0_data.add_action(Action::WorkStart(WorkStart {
        ts: Time::hm(10, 15),
        task: JiraIssue::create("A-1").unwrap(),
        description: "Description1".to_string(),
    }));
    db.store_day(&day0_data).unwrap();

    let reloaded = db.get_day(*DAY0).unwrap();
    assert_eq!(reloaded, day0_data);
}

#[test]
fn test_load_previous_day() {
    let db = TmpDB::new();

    let mut day0_data = ActiveDay::new(*DAY0, Location::Office, None);
    day0_data.add_action(Action::WorkStart(WorkStart {
        ts: Time::hm(10, 15),
        task: JiraIssue::create("A-1").unwrap(),
        description: "Description1".to_string(),
    }));
    db.store_day(&day0_data).unwrap();

    let next_day = db.get_day(DAY0.next(&SimpleDayForwarder)).unwrap();
    assert_eq!(
        next_day.active_issue(),
        Some(&JiraIssue {
            ident: "A-1".to_string(),
            description: None,
            default_action: Some("Description1".to_string()),
        })
    );
}

#[test]
fn store_load_active_day() {
    let orig = ActiveDayBuilder {
        day: *DAY0,
        main_location: Location::Home,
        active_issue: Some(JiraIssue {
            ident: "A-123".to_string(),
            description: Some("active description".to_string()),
            default_action: Some("active default action".to_string()),
        }),
        actions: vec![
            day_start("h 9"),
            issue_start("9", "A-234", "issue 234", "development"),
            issue_end("10", "A-234"),
            work("11", "12", "I-15", "some stuff"),
            day_end("17:59"),
        ],
    }
    .build();

    let db = TmpDB::new();
    db.store_day(&orig).unwrap();

    let loaded = db.load_day(orig.get_day()).unwrap();
    assert_eq!(loaded, Some(orig));
}

#[test]
fn day_collection_testing() -> DBResult<()> {
    let db = TmpDB::new();
    let days: Vec<Day> = (*DAY0).iter(WeekDayForwarder).take(30).collect();
    let mut original = Vec::new();
    for day in &days {
        let ad = build_test_day(*day);
        db.store_day(&ad)?;
        original.push(ad);
    }

    let listed = db.list_days(..).unwrap();
    assert_eq!(listed, days);

    let to_load = db.list_days((*DAY0 + 3)..)?;
    assert_eq!(&to_load[..], &days[2..]);
    let loaded: Result<Vec<_>, _> = to_load.iter().map(|day| db.load_day(*day)).collect();
    let loaded: Vec<ActiveDay> = loaded.unwrap().into_iter().flatten().collect();
    assert_eq!(&loaded[..], &original[2..]);

    Ok(())
}

#[test]
fn store_load_recent() {
    let timeline = DefaultTimeline;
    let db = TmpDB::new();
    let empty = RecentIssuesData::default();
    db.store_recent(&empty).unwrap();
    assert_eq!(db.load_recent().unwrap(), empty);

    let with_entries = RecentIssuesData {
        issues: vec![
            RecentIssue {
                issue: JiraIssue::create("R-453433").unwrap(),
                last_used: timeline.now(),
            },
            RecentIssue {
                last_used: timeline.now(),
                issue: JiraIssue {
                    ident: "REISSUE-4435432".to_string(),
                    description: Some("some \n jira \t description äö¬½a stuff".to_string()),
                    default_action: Some("@#+~ß§æs".to_string()),
                },
            },
        ],
    };

    db.store_recent(&with_entries).unwrap();
    assert_eq!(db.load_recent().unwrap(), with_entries)
}

fn build_test_day(day: Day) -> ActiveDay {
    let cd: chrono::NaiveDate = day.into();
    let day_str = format!("{}{}{}", cd.year(), cd.month(), cd.day());

    ActiveDayBuilder {
        day,
        main_location: Location::Office,
        active_issue: None,
        actions: vec![
            day_start("9"),
            issue_start("9", &format!("AT-{}", day_str), "description", "dev"),
            work("10", "11", "M-102", "meeting"),
            day_end("17"),
        ],
    }
    .build()
}
