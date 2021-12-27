#![allow(dead_code)]

use crate::parsing::time::Time;
use std::fmt::Debug;
use std::time::SystemTime;

pub fn now() -> chrono::NaiveDateTime {
    chrono::DateTime::<chrono::Local>::from(SystemTime::now()).naive_local()
}

pub fn time_now() -> chrono::NaiveTime {
    now().time()
}

pub fn today() -> chrono::NaiveDate {
    now().date()
}

#[derive(Clone, Debug)]
pub struct DefaultTimeline;

impl Timeline for DefaultTimeline {
    fn now(&self) -> chrono::NaiveDateTime {
        crate::util::now()
    }
}

#[derive(Clone, Debug)]
pub struct StaticTimeline(chrono::NaiveDateTime);

impl StaticTimeline {
    pub fn parse(s: &str) -> StaticTimeline {
        StaticTimeline(chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").unwrap())
    }
}

impl Timeline for StaticTimeline {
    fn now(&self) -> chrono::NaiveDateTime {
        self.0
    }
}

pub trait Timeline: Clone + Debug + Send + Sync {
    fn now(&self) -> chrono::NaiveDateTime;

    fn time_now(&self) -> Time {
        let now = self.now();
        now.time().into()
    }

    fn today(&self) -> chrono::NaiveDate {
        self.now().date()
    }
}
