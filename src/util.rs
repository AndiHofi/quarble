#![allow(dead_code)]

use crate::data::Day;
use crate::parsing::time::Time;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[deprecated]
pub fn now() -> chrono::NaiveDateTime {
    DefaultTimeline.now()
}

#[deprecated]
pub fn time_now() -> chrono::NaiveTime {
    DefaultTimeline.now().time()
}

#[derive(Debug)]
pub struct DefaultTimeline;

impl TimelineProvider for DefaultTimeline {
    fn now(&self) -> chrono::NaiveDateTime {
        chrono::DateTime::<chrono::Local>::from(SystemTime::now()).naive_local()
    }
}

#[derive(Debug)]
pub struct StaticTimeline(Arc<Mutex<chrono::NaiveDateTime>>);

impl StaticTimeline {
    pub fn parse(s: &str) -> StaticTimeline {
        StaticTimeline(Arc::new(Mutex::new(
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").unwrap(),
        )))
    }

    pub fn advance(&self) {
        let mut guard = self.0.lock().unwrap();
        *guard += chrono::Duration::minutes(1);
    }
}

impl TimelineProvider for StaticTimeline {
    fn now(&self) -> chrono::NaiveDateTime {
        *self.0.lock().unwrap()
    }
}

pub trait TimelineProvider: Debug + Send + Sync {
    fn now(&self) -> chrono::NaiveDateTime;

    fn time_now(&self) -> Time {
        let now = self.now();
        now.time().into()
    }

    fn naive_now(&self) -> chrono::NaiveTime {
        self.now().time()
    }

    fn today(&self) -> Day {
        self.now().date().into()
    }
}

pub type Timeline = Arc<dyn TimelineProvider>;
