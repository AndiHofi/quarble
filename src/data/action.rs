use chrono::NaiveTime;

use crate::data::location::Location;
use crate::data::work::{Work, WorkEnd, WorkEvent, WorkStart};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Action {
    Work(Work),
    WorkEvent(WorkEvent),
    WorkStart(WorkStart),
    WorkEnd(WorkEnd),
    DayStart(DayStart),
    DayEnd(DayEnd),
    DayOff,
    ZA(ZA),
    Vacation,
    Sick,
    Doctor(Doctor),
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ZA {
    pub start: chrono::NaiveTime,
    pub end: chrono::NaiveTime,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DayStart {
    pub location: Location,
    pub ts: NaiveTime,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct DayEnd {
    pub location: Location,
    pub ts: NaiveTime,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Doctor {
    pub start: chrono::NaiveTime,
    pub end: chrono::NaiveTime,
}
