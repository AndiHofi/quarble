use crate::data::JiraIssue;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct Work {
    pub start: chrono::NaiveTime,
    pub end: chrono::NaiveTime,
    pub task: JiraIssue,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WorkEvent {
    pub ts: chrono::NaiveTime,
    pub task: JiraIssue,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WorkStart {
    pub ts: chrono::NaiveTime,
    pub task: JiraIssue,
    pub description: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct WorkEnd {
    pub ts: chrono::NaiveTime,
    pub task: JiraIssue,
}
