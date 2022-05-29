use crate::data::JiraIssue;
use crate::parsing::time::Time;

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct CurrentWork {
    pub start: Time,
    pub task: JiraIssue,
    pub description: String,
}