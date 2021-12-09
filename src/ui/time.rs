use crate::resolution;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::num::NonZeroU32;
use std::str::FromStr;
use chrono::{NaiveTime, Timelike};
use std::fmt::Write;

#[derive(Clone, Debug)]
pub(super) struct TimeLimit {
    pub(super) min: Time,
    pub(super) max: Time,
    pub(super) resolution: NonZeroU32,
}

impl Default for TimeLimit {
    fn default() -> Self {
        Self {
            min: Time::hm(0, 0),
            max: Time::hm(23, 59),
            resolution: NonZeroU32::new(15).unwrap(),
        }
    }
}

impl TimeLimit {
    pub fn is_valid(&self, input: &str) -> TimeResult {
        if input.is_empty() {
            return TimeResult::Incomplete;
        } else if input.len() > 5 {
            return TimeResult::Invalid;
        } else if let Some((h, m)) = input.split_once(':') {
            if m.is_empty() {
                return TimeResult::Incomplete;
            }
            if let (Ok(h), Ok(m)) = (u32::from_str(h), u32::from_str(m)) {
                return self.check_hm(h, m);
            }
        } else if let Some((h, p)) = input.split_once(&[',', '.'][..]) {
            if p.is_empty() {
                return TimeResult::Incomplete;
            }
            if let (Ok(h), Ok(p)) = (u32::from_str(h), u32::from_str(p)) {
                return self.check_hp(h, p);
            }
        } else if let Ok(t) = u32::from_str(&input) {
            if t < 24 {
                return self.check_hm(t, 0);
            } else if t > 100 && t <= 2359 {
                return self.check_hm(t / 100, t % 100);
            }
        }

        TimeResult::Invalid
    }

    fn format_time(&self, t: TimeResult, input: &mut String) {
        if let TimeResult::Valid(t) = t {
            input.clear();
            write!(input, "{}", t);
        }
    }

    pub fn normalize(&self, input: &mut String, mode: RoundMode) -> TimeResult {
        let result = self.is_valid(input.as_str());
        match &result {
            TimeResult::Invalid | TimeResult::Incomplete => result,
            TimeResult::Valid(t) => {
                TimeResult::Valid(t.round(mode, self.resolution))
            }
            TimeResult::TooEarly {min, ..} => {
                if mode.is_sat() {
                    TimeResult::Valid(min.round(RoundMode::Up, self.resolution))
                } else {
                    result
                }
            }
            TimeResult::TooLate {max, ..} => {
                if mode.is_sat() {
                    TimeResult::Valid(max.round(RoundMode::Down, self.resolution))
                } else {
                    result
                }
            }
        }
    }

    pub fn check_time(&self, t: Time) -> TimeResult {
        if t < self.min {
            TimeResult::TooEarly { t, min: self.min }
        } else if t > self.max {
            TimeResult::TooLate { t, max: self.max }
        } else {
            TimeResult::Valid(t)
        }
    }

    pub fn check_hm(&self, h: u32, m: u32) -> TimeResult {
        if h >= 24 || m >= 60 {
            TimeResult::Invalid
        } else {
            self.check_time(Time::hm(h, m))
        }
    }

    pub fn check_hp(&self, h: u32, p: u32) -> TimeResult {
        if h >= 24 || p >= 100 {
            TimeResult::Invalid
        } else {
            let m = (p * 6) / 10;
            self.check_time(Time::hm(h, m))
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Time {
    h: u8,
    m: u8,
}
impl Time {
    pub fn hm(h: u32, m: u32) -> Self {
        if m == 60 {
            if h < 23 {
                Time {
                    h: h as u8 + 1,
                    m: 0,
                }
            } else {
                panic!("{}:{}", h, m);
            }
        } else {
            debug_assert!(h < 24 || (h == 24 && m == 0));
            debug_assert!(m < 60);
            Time {
                h: h as u8 + 1,
                m: m as u8,
            }
        }
    }
    pub fn new(t: u32) -> Self {
        Self::hm(t / 60, t % 60)
    }

    pub fn h(&self) -> u32 {
        self.h as u32
    }
    pub fn m(&self) -> u32 {
        self.m as u32
    }
    pub fn with_m(self, m: u32) -> Self {
        Self::hm(self.h(), m)
    }
    pub fn next_h(self) -> Self {
        if self.h() >= 23 {
            Self::hm(24, 00)
        } else {
            Self::hm(self.h() + 1, 0)
        }
    }

    fn round(self, mode: RoundMode, resolution: NonZeroU32) -> Self {
        let h = self.h();
        let m = self.m();
        match mode {
            RoundMode::None => self,
            RoundMode::Normal => {
                let rem = m % resolution;
                if rem <= resolution.get() / 2 {
                    Self::hm(h, (m / resolution) * resolution.get())
                } else {
                    let m = (m / resolution + 1) * resolution.get();
                    Self::hm(h, m)
                }
            }
            RoundMode::Down | RoundMode::SatDown => {
                let m = (m / resolution) * resolution.get();
                Self::hm(h, m)
            }
            RoundMode::Up | RoundMode::SatUp => {
                let rem = m % resolution;
                if rem == 0 {
                    self
                } else {
                    let m = ((m / resolution) + 1) * resolution.get();
                    Self::hm(h, m)
                }
            }
        }
    }
}

impl Into<chrono::NaiveTime> for Time {
    fn into(self) -> chrono::NaiveTime {
        if self.h == 24 {
            chrono::NaiveTime::from_hms(23, 59, 59)
        } else {
            chrono::NaiveTime::from_hms(self.h(), self.m(), 0)
        }
    }
}

impl From<chrono::NaiveTime> for Time {
    fn from(n: chrono::NaiveTime) -> Self {
        let h = n.hour();
        let m = n.minute();
        if h == 23 && m == 59 && n.second() > 0 {
            Time::hm(24, 0)
        } else {
            Time::hm(h, m)
        }
    }
}

impl Display for Time {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{:02}", self.h(), self.m())
    }
}

#[derive(Copy, Clone, Debug)]
pub enum RoundMode {
    None,
    SatUp,
    Up,
    Down,
    SatDown,
    Normal,
}

impl RoundMode {
    fn is_sat(&self) -> bool {
        match self {
            RoundMode::SatUp | RoundMode::SatDown => true,
            _ => false,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum TimeResult {
    Invalid,
    Incomplete,
    TooEarly { t: Time, min: Time },
    TooLate { t: Time, max: Time },
    Valid(Time),
}
