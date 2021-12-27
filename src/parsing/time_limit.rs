use crate::parsing::parse_result::ParseResult;
use crate::parsing::round_mode::RoundMode;
use crate::parsing::time::Time;
use std::fmt::Write;
use std::num::NonZeroU8;
use std::str::FromStr;

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
            return TimeResult::Invalid(InvalidTime::Bad);
        } else if let Some((h, m)) = input.split_once(':') {
            if m.starts_with('+') {
                return TimeResult::Invalid(InvalidTime::Bad);
            }
            if m.is_empty() {
                return TimeResult::Incomplete;
            }
            if let (Ok(h), Ok(m)) = (u32::from_str(h), u32::from_str(m)) {
                return self.check_hm(h, m);
            }
        } else if let Some((h, p)) = input.split_once(&[',', '.'][..]) {
            if p.starts_with('+') {
                return TimeResult::Invalid(InvalidTime::Bad);
            }
            if p.is_empty() {
                return TimeResult::Incomplete;
            }
            if let (Ok(h), Ok(p)) = (u32::from_str(h), u32::from_str(p)) {
                return self.check_hp(h, p);
            }
        } else if let Ok(t) = u32::from_str(input) {
            if t < 24 {
                return self.check_hm(t, 0);
            } else if t > 100 && t <= 2359 {
                return self.check_hm(t / 100, t % 100);
            }
        }

        TimeResult::Invalid(InvalidTime::Bad)
    }

    fn format_time(&self, t: TimeResult, input: &mut String) {
        if let TimeResult::Valid(t) = t {
            input.clear();
            write!(input, "{}", t).unwrap();
        }
    }

    pub fn normalize(&self, input: &mut String, mode: RoundMode) -> TimeResult {
        let result = self.is_valid(input.as_str());
        match result {
            TimeResult::Valid(t) => TimeResult::Valid(t.round(mode, self.resolution.into())),
            TimeResult::Invalid(InvalidTime::TooEarly { min, .. }) => {
                if mode.is_sat() {
                    TimeResult::Valid(min.round(RoundMode::Up, self.resolution.into()))
                } else {
                    result
                }
            }
            TimeResult::Invalid(InvalidTime::TooLate { max, .. }) => {
                if mode.is_sat() {
                    TimeResult::Valid(max.round(RoundMode::Down, self.resolution.into()))
                } else {
                    result
                }
            }
            r @ (TimeResult::None | TimeResult::Invalid(_) | TimeResult::Incomplete) => r,
        }
    }

    pub fn check_time(&self, t: Time) -> TimeResult {
        if t < self.min {
            TimeResult::Invalid(InvalidTime::TooEarly { t, min: self.min })
        } else if t > self.max {
            TimeResult::Invalid(InvalidTime::TooLate { t, max: self.max })
        } else {
            TimeResult::Valid(t)
        }
    }

    pub fn check_hm(&self, h: u32, m: u32) -> TimeResult {
        if h >= 24 || m >= 60 {
            TimeResult::Invalid(InvalidTime::Bad)
        } else {
            self.check_time(Time::hm(h, m))
        }
    }

    pub fn check_hp(&self, h: u32, p: u32) -> TimeResult {
        if h >= 24 || p >= 100 {
            TimeResult::Invalid(InvalidTime::Bad)
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

pub fn check_limits(t: Time, limits: &[TimeLimit]) -> TimeResult {
    for limit in limits {
        match limit.check_time(t) {
            ParseResult::Invalid(_) => (),
            r => return r,
        }
    }

    ParseResult::Invalid(InvalidTime::Bad)
}

#[derive(Debug, Copy, Clone)]
pub enum InvalidTime {
    Bad,
    TooEarly { t: Time, min: Time },
    TooLate { t: Time, max: Time },
}

pub type TimeResult = ParseResult<Time, InvalidTime>;
