use crate::data::JiraIssue;
use crate::parsing::time::Time;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Work {
    pub start: Time,
    pub end: Time,
    pub task: JiraIssue,
    pub description: String,
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
