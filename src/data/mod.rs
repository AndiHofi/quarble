use crate::parsing::time::Time;
pub use action::{Action, DayEnd, DayStart, Doctor, TimedAction, ZA};
pub use day::{Day, DayForwarder, WeekDayForwarder};
pub use jira_issue::JiraIssue;
pub use location::Location;
use std::collections::BTreeSet;
pub use work::{Work, WorkEnd, WorkEvent, WorkStart};

mod action;
mod day;
mod day_normalizer;
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

    actions: BTreeSet<Action>,
}

lazy_static::lazy_static! {
    static ref NO_ACTIONS_INT: BTreeSet<Action> = BTreeSet::new();
}

impl ActiveDay {
    pub fn no_action() -> &'static BTreeSet<Action> {
        &NO_ACTIONS_INT
    }

    pub fn new(day: Day, main_location: Location, active_issue: Option<JiraIssue>) -> ActiveDay {
        ActiveDay {
            day,
            main_location,
            active_issue,
            actions: BTreeSet::new(),
        }
    }

    pub fn get_day(&self) -> Day {
        self.day
    }

    /// The issue that was active when starting the day
    pub fn active_issue(&self) -> Option<&JiraIssue> {
        self.active_issue.as_ref()
    }

    pub fn main_location(&self) -> &Location {
        &self.main_location
    }

    pub fn actions(&self) -> &BTreeSet<Action> {
        &self.actions
    }

    pub fn add_action(&mut self, action: Action) {
        self.actions.insert(action);
    }

    pub fn current_issue(&self, now: Time) -> Option<&JiraIssue> {
        if self
            .actions
            .iter()
            .filter(|e| e.times().0 <= now)
            .rfind(|e| matches![e, Action::WorkEnd(_)])
            .is_some()
        {
            return None;
        }

        if let Some(Action::WorkStart(WorkStart { task, .. })) = self
            .actions
            .iter()
            .filter(|e| e.times().0 <= now)
            .rfind(|e| matches![e, Action::WorkStart(_)])
        {
            Some(task)
        } else {
            self.active_issue.as_ref()
        }
    }
}
