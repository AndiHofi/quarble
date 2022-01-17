use crate::data::*;
use crate::parsing;
use crate::parsing::time::Time;
use crate::parsing::time_limit::TimeLimit;
use crate::ui::fast_day_start::DayStartBuilder;
use crate::util::{DefaultTimeline, TimelineProvider};
use std::sync::Arc;

pub fn workn(start: &str, end: &str, issue: &str, description: &str) -> Work {
    Work {
        start: time(start),
        end: time(end),
        task: JiraIssue::create(issue).unwrap(),
        description: description.to_string(),
    }
}

pub fn work(start: &str, end: &str, issue: &str, description: &str) -> Action {
    Action::Work(workn(start, end, issue, description))
}

pub fn time(time: &str) -> Time {
    Time::parse_prefix(time).0.get().unwrap()
}

pub fn issue_start(start: &str, issue: &str, description: &str, action: &str) -> Action {
    Action::WorkStart(WorkStart {
        ts: time(start),
        task: JiraIssue {
            ident: issue.to_string(),
            description: if description.is_empty() {
                None
            } else {
                Some(description.to_string())
            },
            default_action: None,
        },
        description: action.to_string(),
    })
}

pub fn issue_end(end: &str, issue: &str) -> Action {
    Action::WorkEnd(WorkEnd {
        ts: time(end),
        task: JiraIssue::create(issue).unwrap(),
    })
}

pub fn day_start(input: &str) -> Action {
    let timeline: Arc<dyn TimelineProvider> = Arc::new(DefaultTimeline);
    let mut builder = DayStartBuilder::default();
    builder.parse_value(&timeline, &[TimeLimit::default()], input);
    builder.try_build(&timeline).map(Action::DayStart).unwrap()
}

pub fn day_end(input: &str) -> Action {
    let timeline = Arc::new(DefaultTimeline);
    parsing::parse_day_end(timeline.time_now(), input)
        .get()
        .map(|ts| Action::DayEnd(DayEnd { ts }))
        .unwrap()
}
