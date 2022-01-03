use crate::data::JiraIssue;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

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

impl Action {
    pub fn as_no_time(&self) -> NoTimeDisplay {
        NoTimeDisplay(self)
    }

    pub fn start(&self) -> Option<Time> {
        match self {
            Action::Work(w) => Some(w.start.into()),
            Action::WorkEvent(w) => Some(w.ts.into()),
            Action::WorkStart(w) => Some(w.ts.into()),
            Action::DayStart(w) => Some(w.ts.into()),
            Action::ZA(w) => Some(w.start.into()),
            Action::Doctor(w) => Some(w.start.into()),
            _ => None,
        }
    }

    pub fn end(&self) -> Option<Time> {
        match self {
            Action::Work(w) => Some(w.end.into()),
            Action::WorkEvent(w) => Some(w.ts.into()),
            Action::WorkEnd(w) => Some(w.ts.into()),
            Action::DayEnd(w) => Some(w.ts.into()),
            Action::ZA(w) => Some(w.end.into()),
            Action::Doctor(w) => Some(w.end.into()),
            _ => None,
        }
    }

    pub fn issue(&self) -> Option<&JiraIssue> {
        match self {
            Action::Work(w) => Some(&w.task),
            Action::WorkEvent(w) => Some(&w.task),
            Action::WorkStart(w) => Some(&w.task),
            Action::WorkEnd(w) => Some(&w.task),
            _ => None,
        }
    }

    pub fn issue_id(&self) -> Option<&str> {
        self.issue().map(|i| i.ident.as_str())
    }
}

impl TimedAction for Action {
    fn times(&self) -> (Time, Option<Time>) {
        let (start, end) = match self {
            Action::Work(Work { start, end, .. }) => (start, Some(end)),
            Action::WorkEvent(WorkEvent { ts, .. }) => (ts, None),
            Action::WorkStart(WorkStart { ts, .. }) => (ts, None),
            Action::WorkEnd(WorkEnd { ts, .. }) => (ts, None),
            Action::DayStart(DayStart { ts, .. }) => (ts, None),
            Action::DayEnd(DayEnd { ts, .. }) => (ts, None),
            Action::DayOff | Action::Vacation | Action::Sick => (&Time::ZERO, None),
            Action::ZA(ZA { start, end }) => (start, Some(end)),
            Action::Doctor(Doctor { start, end }) => (start, Some(end)),
        };
        (*start, end.cloned())
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

pub struct NoTimeDisplay<'a>(pub &'a Action);

impl<'a> Display for NoTimeDisplay<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            Action::Work(w) => {
                write!(f, "{}", w.description)
            }
            Action::WorkEvent(e) => {
                write!(f, "{}", e.description)
            }
            Action::WorkStart(s) => {
                write!(f, "{}", s.description)
            }
            Action::WorkEnd(_) => {
                write!(f, "end")
            }
            Action::DayStart(s) => {
                write!(f, "{} start", s.location)
            }
            Action::DayEnd(_) => {
                write!(f, "work end")
            }
            Action::DayOff => {
                write!(f, "Day off")
            }
            Action::ZA(_) => {
                write!(f, "time off")
            }
            Action::Vacation => {
                write!(f, "Vacation")
            }
            Action::Sick => {
                write!(f, "Sick leave")
            }
            Action::Doctor(_) => {
                write!(f, "doctor")
            }
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Work(w) => {
                write!(
                    f,
                    "{} - {} | {}: {}",
                    w.start, w.end, w.task.ident, w.description
                )
            }
            Action::WorkEvent(e) => {
                write!(f, "at {} | {}: {}", e.ts, e.task.ident, e.description)
            }
            Action::WorkStart(s) => {
                write!(f, "{} -   | {}: {}", s.ts, s.task.ident, s.description)
            }
            Action::WorkEnd(e) => {
                write!(f, "   - {} | {}", e.ts, e.task.ident)
            }
            Action::DayStart(s) => {
                write!(f, "{} -   | {} start", s.ts, s.location)
            }
            Action::DayEnd(e) => {
                write!(f, "   - {} | work end", e.ts)
            }
            Action::DayOff => {
                write!(f, "Day off")
            }
            Action::ZA(z) => {
                write!(f, "{} - {} | time off", z.start, z.end)
            }
            Action::Vacation => {
                write!(f, "Vacation")
            }
            Action::Sick => {
                write!(f, "Sick leave")
            }
            Action::Doctor(d) => {
                write!(f, "{} - {} | doctor", d.start, d.end)
            }
        }
    }
}

pub trait TimedAction {
    fn times(&self) -> (Time, Option<Time>);
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ZA {
    pub start: Time,
    pub end: Time,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DayStart {
    pub location: Location,
    pub ts: Time,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct DayEnd {
    pub ts: Time,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Doctor {
    pub start: Time,
    pub end: Time,
}
