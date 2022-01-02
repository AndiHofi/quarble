pub use action::{Action, DayEnd, DayStart, Doctor, TimedAction, ZA};
pub use day::{Day, DayForwarder, WeekDayForwarder};
pub use jira_issue::JiraIssue;
pub use location::Location;
pub use work::{Work, WorkEnd, WorkEvent, WorkStart};

mod action;
mod day;
mod jira_issue;
mod location;
mod work;
mod work_day;

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct ActiveDay {
    day: Day,
    main_location: Location,
    /// The jira issue that had a start event in previous days, but never ended
    active_issue: Option<JiraIssue>,

    actions: Vec<Action>,
}

impl ActiveDay {
    pub fn new(day: Day, main_location: Location, active_issue: Option<JiraIssue>) -> ActiveDay {
        ActiveDay {
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
