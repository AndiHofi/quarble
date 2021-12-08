use crate::data::task::Task;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Work {
    pub start: chrono::NaiveTime,
    pub end: chrono::NaiveTime,
    pub task: Task,
    pub description: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct WorkEvent {
    pub ts: chrono::NaiveTime,
    pub task: Task,
    pub description: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct WorkStart {
    pub ts: chrono::NaiveTime,
    pub task: Task,
    pub description: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct WorkEnd {
    pub ts: chrono::NaiveTime,
    pub task: Task,
}