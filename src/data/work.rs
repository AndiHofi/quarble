use crate::data::{JiraIssue, TimedAction};
use crate::parsing::time::Time;
use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Work {
    pub start: Time,
    pub end: Time,
    pub task: JiraIssue,
    pub description: String,
}

impl PartialOrd<Self> for Work {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(TimedAction::cmp(self, other))
    }
}

impl Ord for Work {
    fn cmp(&self, other: &Self) -> Ordering {
        TimedAction::cmp(self, other)
    }
}

impl TimedAction for Work {
    fn times(&self) -> (Time, Option<Time>) {
        (self.start, Some(self.end))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WorkEvent {
    pub ts: Time,
    pub task: JiraIssue,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WorkStart {
    pub ts: Time,
    pub task: JiraIssue,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WorkEnd {
    pub ts: Time,
    pub task: JiraIssue,
}
