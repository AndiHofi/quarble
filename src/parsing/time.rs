use std::fmt::Write;
use std::fmt::{Display, Formatter};
use std::num::{NonZeroU32, NonZeroU8};
use std::str::FromStr;

use chrono::Timelike;

use crate::parsing::time_relative::TimeRelative;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TimeLimit {
    min: Time,
    max: Time,
    resolution: NonZeroU8,
}

impl Default for TimeLimit {
    fn default() -> Self {
        Self {
            min: Time::hm(0, 0),
            max: Time::hm(23, 59),
            resolution: NonZeroU8::new(15).unwrap(),
        }
    }
}

impl TimeLimit {
    pub const EMPTY: Self = Self {
        min: Time::ZERO,
        max: Time::ZERO,
        resolution: unsafe { NonZeroU8::new_unchecked(1) },
    };

    pub fn simple(min: Time, max: Time) -> Self {
        Self::new(min, max, 1)
    }

    pub fn new(min: Time, max: Time, resolution: u8) -> Self {
        if min >= max {
            Self::EMPTY
        } else {
            let resolution = NonZeroU8::new(resolution).unwrap();
            Self {
                min,
                max,
                resolution,
            }
        }
    }

    pub fn min(&self) -> Time {
        self.min
    }

    pub fn max(&self) -> Time {
        self.max
    }

    pub fn resolution(&self) -> NonZeroU8 {
        self.resolution
    }

    pub fn is_valid(&self, input: &str) -> TimeResult {
        if input.is_empty() {
            return TimeResult::Incomplete;
        } else if input.len() > 5 || input.starts_with('+') {
            return TimeResult::Invalid;
        } else if let Some((h, m)) = input.split_once(':') {
            if m.starts_with('+') {
                return TimeResult::Invalid;
            }
            if m.is_empty() {
                return TimeResult::Incomplete;
            }
            if let (Ok(h), Ok(m)) = (u32::from_str(h), u32::from_str(m)) {
                return self.check_hm(h, m);
            }
        } else if let Some((h, p)) = input.split_once(&[',', '.'][..]) {
            if p.starts_with('+') {
                return TimeResult::Invalid;
            }
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
            write!(input, "{}", t).unwrap();
        }
    }

    pub fn normalize(&self, input: &mut String, mode: RoundMode) -> TimeResult {
        let result = self.is_valid(input.as_str());
        match &result {
            TimeResult::Invalid | TimeResult::Incomplete => result,
            TimeResult::Valid(t) => TimeResult::Valid(t.round(mode, self.resolution.into())),
            TimeResult::TooEarly { min, .. } => {
                if mode.is_sat() {
                    TimeResult::Valid(min.round(RoundMode::Up, self.resolution.into()))
                } else {
                    result
                }
            }
            TimeResult::TooLate { max, .. } => {
                if mode.is_sat() {
                    TimeResult::Valid(max.round(RoundMode::Down, self.resolution.into()))
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

    pub fn split_at(&self, time: Time) -> (TimeLimit, TimeLimit) {
        (self.with_max(time), self.with_min(time))
    }

    pub fn split(&self, exclude: TimeLimit) -> (TimeLimit, TimeLimit) {
        (self.with_max(exclude.min), self.with_min(exclude.max))
    }

    pub fn with_min(self, new_min: Time) -> Self {
        if new_min >= self.max {
            TimeLimit::EMPTY
        } else {
            TimeLimit {
                min: new_min,
                max: self.max,
                resolution: self.resolution,
            }
        }
    }

    pub fn with_max(self, new_max: Time) -> Self {
        if new_max <= self.min {
            TimeLimit::EMPTY
        } else {
            TimeLimit {
                min: self.min,
                max: new_max,
                resolution: self.resolution,
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Time {
    h: u8,
    m: u8,
}
impl Time {
    pub const ZERO: Time = Time::hm(0, 0);
    pub const fn hm(h: u32, m: u32) -> Self {
        if m == 60 {
            if h < 23 {
                Time {
                    h: h as u8 + 1,
                    m: 0,
                }
            } else {
                panic!("Invalid time");
            }
        } else {
            debug_assert!(h < 24 || (h == 24 && m == 0));
            debug_assert!(m < 60);
            Time {
                h: h as u8,
                m: m as u8,
            }
        }
    }
    pub fn new(t: u32) -> Self {
        Self::hm(t / 60, t % 60)
    }

    pub fn try_add_relative(self, tr: TimeRelative) -> Option<Self> {
        let mut h = self.h as i32 + tr.offset_hours();
        let mut m = self.m as i32 + tr.offset_minutes();
        if m < 0 {
            m += 60;
            h -= 1;
        } else if m >= 60 {
            m -= 60;
            h += 1;
        }
        if h < 0 || h >= 24 || m < 0 || m >= 60 {
            None
        } else {
            Some(Time::hm(h as u32, m as u32))
        }
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

impl From<&chrono::NaiveTime> for Time {
    fn from(n: &chrono::NaiveTime) -> Self {
        From::<chrono::NaiveTime>::from(*n)
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

#[cfg(test)]
mod test {
    use crate::parsing::time::Time;
    use crate::parsing::time_relative::TimeRelative;

    #[test]
    fn sub_time() {
        let time = Time::hm(1, 0);
        assert_eq!(
            time.try_add_relative(TimeRelative::new(true, 0, 1).unwrap()),
            Some(Time::hm(0, 59))
        );
    }
}
