use crate::data::day_normalizer::FilledRange;
use crate::data::{JiraIssue, TimedAction, Work};
use crate::parsing::time::Time;
use crate::parsing::time_limit::TimeRange;
use crate::parsing::time_relative::TimeRelative;
use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct We {
    pub id: String,
    pub description: String,
    pub start: Time,
    pub end: Time,
    pub implicit: bool,
}

pub trait HasRange {
    fn range(&self) -> TimeRange;
}

impl HasRange for We {
    fn range(&self) -> TimeRange {
        TimeRange::new(self.start, self.end)
    }
}

impl HasRange for FilledRange {
    fn range(&self) -> TimeRange {
        self.range
    }
}

impl PartialOrd for We {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(TimedAction::cmp(self, other))
    }
}

impl Ord for We {
    fn cmp(&self, other: &Self) -> Ordering {
        TimedAction::cmp(self, other)
    }
}

impl TimedAction for We {
    fn times(&self) -> (Time, Option<Time>) {
        (self.start, Some(self.end))
    }
}

impl We {
    pub fn same_issue(&self, other: &Self) -> bool {
        self.id == other.id
    }

    pub fn range(&self) -> TimeRange {
        TimeRange::new(self.start, self.end)
    }

    pub fn duration(&self) -> TimeRelative {
        (self.end - self.start).abs()
    }
}

impl From<We> for Work {
    fn from(w: We) -> Self {
        Work {
            start: w.start,
            end: w.end,
            task: JiraIssue {
                ident: w.id,
                description: None,
                default_action: None,
            },
            description: w.description,
        }
    }
}
