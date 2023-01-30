#![allow(dead_code)]

use crate::data::Day;
use crate::parsing::time::Time;
use crate::ui::Message;
use arc_swap::ArcSwap;
use iced_futures::BoxFuture;
use iced_native::command::Action;
use iced_native::Command;
use std::fmt::Debug;
use std::future::Future;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
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

pub fn update_arcswap<A, F, D>(orig: &D, f: F)
where
    A: Clone,
    F: FnOnce(&mut A),
    D: Deref<Target = ArcSwap<A>>,
{
    let target = orig.deref();
    let mut updating = (**target.load()).clone();
    f(&mut updating);
    target.store(Arc::new(updating))
}

impl From<StaticTimeline> for Timeline {
    fn from(t: StaticTimeline) -> Self {
        Arc::new(t)
    }
}

enum MFuture<T: Send> {
    Ready(Pin<T>),
    Polled,
}

pub fn msg(m: Message) -> Command<Message> {
    let future = async move {m};

    Command::single(Action::Future(Box::pin(future)))
}
