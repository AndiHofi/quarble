pub use action::{Action, TimedAction, DayEnd, DayStart, Doctor, ZA};
pub use day::{Day, DayForwarder, WeekDayForwarder};
pub use jira_issue::JiraIssue;
pub use location::Location;
pub use task::Task;
pub use work::{Work, WorkEnd, WorkEvent, WorkStart};

mod action;
mod day;
mod jira_issue;
mod location;
mod task;
mod work;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct WorkDay {
    day: Day,
    main_location: Location,
    /// The jira issue that had a start event in previous days, but never ended
    active_issue: Option<JiraIssue>,

    actions: Vec<Action>,
}

impl WorkDay {
    pub fn new(day: Day, main_location: Location, active_issue: Option<JiraIssue>) -> WorkDay {
        WorkDay {
            day,
            main_location,
            active_issue,
            actions: Vec::new(),
        }
    }

    pub fn get_day(&self) -> Day {
        self.day
    }

    pub fn active_issue(&self) -> Option<&JiraIssue> {
        self.active_issue.as_ref()
    }

    pub fn main_location(&self) -> &Location {
        &self.main_location
    }

    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    pub fn add_action(&mut self, action: Action) {
        self.actions.push(action);
    }
}
