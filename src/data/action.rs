use chrono::NaiveTime;

use crate::data::location::Location;
use crate::data::work::{Work, WorkEnd, WorkEvent, WorkStart};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Action {
    Work(Work),
    WorkEvent(WorkEvent),
    WorkStart(WorkStart),
    WorkEnd(WorkEnd),
    DayStart {
        location: Location,
        ts: NaiveTime,
    },
    DayEnd {
        location: Location,
        ts: NaiveTime,
    },
    DayOff,
    ZA {
        start: chrono::NaiveTime,
        end: chrono::NaiveTime,
    },
    Vacation,
    Sick,
    Doctor {
        start: chrono::NaiveTime,
        end: chrono::NaiveTime,
    },
}