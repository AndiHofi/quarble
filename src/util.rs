#![allow(dead_code)]

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
