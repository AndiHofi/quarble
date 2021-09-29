use crate::data::jira_issue::JiraIssue;
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum Task {
    Jira(JiraIssue),
    Meeting,
    Admin,
}