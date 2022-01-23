use crate::data::{Action, Day, JiraIssue, Location, TimedAction, WorkStart};
use crate::parsing::time::Time;
use std::collections::BTreeSet;

pub struct ActiveDayBuilder {
    pub day: Day,
    pub main_location: Location,
    /// The jira issue that had a start event in previous days, but never ended
    pub active_issue: Option<JiraIssue>,

    pub actions: Vec<Action>,
}

impl ActiveDayBuilder {
    pub fn build(self) -> ActiveDay {
        let mut r = ActiveDay::new(self.day, self.main_location, self.active_issue);

        for a in self.actions {
            r.add_action(a);
        }
        r
    }
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct ActiveDay {
    day: Day,
    main_location: Location,
    /// The jira issue that had a start event in previous days, but never ended
    active_issue: Option<JiraIssue>,

    actions: BTreeSet<Action>,
}

impl ActiveDay {
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

    pub fn actions_mut(&mut self) -> &mut BTreeSet<Action> {
        &mut self.actions
    }

    pub fn add_action(&mut self, action: Action) {
        self.actions.insert(action);
    }

    pub fn current_issue(&self, now: Time) -> Option<JiraIssue> {
        if self
            .actions
            .iter()
            .filter(|e| e.times().0 <= now)
            .rfind(|e| matches![e, Action::WorkEnd(_)])
            .is_some()
        {
            return None;
        }

        if let Some(Action::WorkStart(WorkStart {
            task, description, ..
        })) = self
            .actions
            .iter()
            .filter(|e| e.times().0 <= now)
            .rfind(|e| matches![e, Action::WorkStart(_)])
        {
            Some(JiraIssue {
                ident: task.ident.clone(),
                default_action: Some(task.default_action.as_ref().unwrap_or(description).clone()),
                description: task.description.clone(),
            })
        } else {
            self.active_issue.clone()
        }
    }

    pub fn last_action_end(&self, now: Time) -> Option<Time> {
        self.actions()
                .iter()
                .filter_map(|t| t.action_end().filter(|end| *end <= now))
                .last()
    }
}
