pub use action::Action;
pub use jira_issue::JiraIssue;
pub use location::Location;
pub use task::Task;
pub use work::{Work, WorkEnd, WorkEvent, WorkStart};

mod location;
mod jira_issue;
mod task;
mod work;
mod action;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct WorkDay {
    main_location: Location,
    /// The jira issue that had a start event in previous days, but never ended
    active_issue: Option<JiraIssue>,

    actions: Vec<Action>,
}

