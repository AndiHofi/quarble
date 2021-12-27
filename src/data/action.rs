use chrono::NaiveTime;
use std::cmp::Ordering;

use crate::data::location::Location;
use crate::data::work::{Work, WorkEnd, WorkEvent, WorkStart};
use crate::parsing::time::Time;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
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

impl TimedAction for Action {
    fn times(&self) -> (Time, Option<Time>) {
        let zero = NaiveTime::from_hms(0, 0, 0);
        let (start, end) = match self {
            Action::Work(Work { start, end, .. }) => (start, Some(end)),
            Action::WorkEvent(WorkEvent { ts, .. }) => (ts, None),
            Action::WorkStart(WorkStart { ts, .. }) => (ts, None),
            Action::WorkEnd(WorkEnd { ts, .. }) => (ts, None),
            Action::DayStart(DayStart { ts, .. }) => (ts, None),
            Action::DayEnd(DayEnd { ts, .. }) => (ts, None),
            Action::DayOff | Action::Vacation | Action::Sick => (&zero, None),
            Action::ZA(ZA { start, end }) => (start, Some(end)),
            Action::Doctor(Doctor { start, end }) => (start, Some(end)),
        };
        (start.into(), end.map(Time::from))
    }
}

impl PartialOrd for Action {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Action {
    fn cmp(&self, other: &Self) -> Ordering {
        let (self_start, self_end) = self.times();
        let (other_start, other_end) = other.times();
        self_start
            .cmp(&other_start)
            .then(match (self_end, other_end) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some(s), Some(o)) => s.cmp(&o),
            })
    }
}

pub trait TimedAction {
    fn times(&self) -> (Time, Option<Time>);
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ZA {
    pub start: chrono::NaiveTime,
    pub end: chrono::NaiveTime,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DayStart {
    pub location: Location,
    pub ts: NaiveTime,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DayEnd {
    pub ts: NaiveTime,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Doctor {
    pub start: chrono::NaiveTime,
    pub end: chrono::NaiveTime,
}
