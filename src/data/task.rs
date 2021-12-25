use crate::data::jira_issue::JiraIssue;
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum Task {
    Jira(JiraIssue),
    Meeting,
    Admin,
}
