use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::time_relative::TimeRelative;
use std::fmt::Write;
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct TimeRange {
    min: Time,
    max: Time,
}

#[allow(dead_code)]
impl TimeRange {
    pub fn new(mut min: Time, max: Time) -> Self {
        if min > max {
            min = max;
        }

        Self { min, max }
    }

    pub fn min(self) -> Time {
        self.min
    }

    pub fn max(self) -> Time {
        self.max
    }

    pub fn is_valid(self, input: &str) -> TimeResult {
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

    fn format_time(self, t: TimeResult, input: &mut String) {
        if let TimeResult::Valid(t) = t {
            input.clear();
            write!(input, "{}", t).unwrap();
        }
    }

    pub fn check_time_overlaps(self, t: Time) -> TimeResult {
        if t < self.min {
            TimeResult::Invalid(InvalidTime::TooEarly { t, min: self.min })
        } else if t > self.max {
            TimeResult::Invalid(InvalidTime::TooLate { t, max: self.max })
        } else {
            TimeResult::Valid(t)
        }
    }

    pub fn check_hm(self, h: u32, m: u32) -> TimeResult {
        if h >= 24 || m >= 60 {
            TimeResult::Invalid(InvalidTime::Bad)
        } else {
            self.check_time_overlaps(Time::hm(h, m))
        }
    }

    pub fn check_hp(self, h: u32, p: u32) -> TimeResult {
        if h >= 24 || p >= 100 {
            TimeResult::Invalid(InvalidTime::Bad)
        } else {
            let m = (p * 6) / 10;
            self.check_time_overlaps(Time::hm(h, m))
        }
    }

    pub fn split_at(&self, time: Time) -> (Self, Self) {
        (self.with_max(time), self.with_min(time))
    }

    pub fn split(&self, exclude: impl Into<TimeRange>) -> (Self, Self) {
        let exclude = exclude.into();
        (self.with_max(exclude.min()), self.with_min(exclude.max()))
    }

    pub fn with_min(self, new_min: Time) -> Self {
        Self {
            min: new_min.min(self.max),
            max: self.max,
        }
    }

    pub fn with_max(self, new_max: Time) -> Self {
        Self {
            min: self.min,
            max: new_max.max(self.min),
        }
    }

    pub fn extend(self, new_bound: Time) -> Self {
        Self {
            min: self.min.min(new_bound),
            max: self.max.max(new_bound),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.min >= self.max
    }

    pub fn duration(&self) -> TimeRelative {
        self.max - self.min
    }

    pub fn contains(self, time: Time) -> bool {
        time >= self.min && time <= self.max
    }

    pub fn overlaps(self, other: TimeRange) -> bool {
        self.contains(other.min)
            || self.contains(other.max)
            || other.contains(self.min)
            || other.contains(self.max)
    }
}

impl Default for TimeRange {
    fn default() -> Self {
        Self {
            min: Time::hm(0, 0),
            max: Time::hm(24, 0),
        }
    }
}

pub fn check_any_limit_overlaps(t: Time, limits: &[TimeRange]) -> TimeResult {
    for limit in limits {
        match limit.check_time_overlaps(t) {
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

#[cfg(test)]
mod test {
    use crate::parsing::time::Time;
    use crate::parsing::time_limit::TimeRange;

    #[test]
    fn test_contains() {
        let t12 = Time::hm(12, 0);
        let t1245 = Time::hm(12, 45);
        let t1230 = Time::hm(12, 30);
        let t1159 = Time::hm(11, 59);
        let t1246 = Time::hm(12, 46);
        let range = TimeRange::new(t12, t1245);

        assert!(range.contains(t12));
        assert!(range.contains(t1230));
        assert!(range.contains(t1245));
        assert!(!range.contains(t1159));
        assert!(!range.contains(t1246));
    }

    #[test]
    fn test_range_overlaps() {
        let range7_10 = TimeRange::new(Time::hm(7, 0), Time::hm(10, 0));
        let range6_8 = TimeRange::new(Time::hm(6, 0), Time::hm(8, 0));
        let range9_11 = TimeRange::new(Time::hm(9, 0), Time::hm(11, 0));
        let range7_8 = TimeRange::new(Time::hm(7, 0), Time::hm(8, 0));
        let range6_7 = TimeRange::new(Time::hm(6, 0), Time::hm(7, 0));
        let range5_6 = TimeRange::new(Time::hm(5, 0), Time::hm(6, 0));
        let range0_24 = TimeRange::new(Time::hm(0, 0), Time::hm(24, 0));
        let range7_7 = TimeRange::new(Time::hm(7, 0), Time::hm(7, 0));
        let range8_1015 = TimeRange::new(Time::hm(8, 0), Time::hm(10, 15));
        let range12_1245 = TimeRange::new(Time::hm(12, 0), Time::hm(12, 45));

        assert!(range0_24.overlaps(range0_24));
        assert!(range0_24.overlaps(range7_10));
        assert!(range7_10.overlaps(range0_24));
        assert!(range7_8.overlaps(range7_10));
        assert!(range7_10.overlaps(range7_8));
        assert!(range7_10.overlaps(range6_8));
        assert!(range7_10.overlaps(range9_11));
        assert!(range7_7.overlaps(range6_8));
        assert!(range6_8.overlaps(range7_7));
        assert!(range7_7.overlaps(range6_7));
        assert!(range7_7.overlaps(range7_8));

        assert!(!range7_8.overlaps(range9_11));
        assert!(!range7_7.overlaps(range5_6));
        assert!(!range5_6.overlaps(range7_7));
        assert!(!range8_1015.overlaps(range12_1245));
    }
}
