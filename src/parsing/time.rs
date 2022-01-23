use std::fmt::{Display, Formatter};
use std::num::NonZeroU32;
use std::ops::{Add, Sub};
use std::str::FromStr;

use crate::parsing::parse_result::ParseResult;
use crate::parsing::rest;
use crate::parsing::round_mode::RoundMode;
use chrono::Timelike;
use regex::{Captures, Regex};
use serde::{Deserializer, Serializer};

use crate::parsing::time_relative::TimeRelative;
use crate::util::Timeline;

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd)]
pub struct Time {
    h: u8,
    m: u8,
}

impl Time {
    pub const ZERO: Time = Time::hm(0, 0);
    pub const MAX: Time = Time::hm(24, 0);
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

    pub fn try_hm(h: u32, m: u32) -> Option<Self> {
        if m == 60 {
            if h < 23 {
                Some(Time {
                    h: h as u8 + 1,
                    m: 0,
                })
            } else {
                None
            }
        } else if (h < 24 || (h == 24 && m == 0)) && m < 60 {
            Some(Time {
                h: h as u8,
                m: m as u8,
            })
        } else {
            None
        }
    }

    pub fn try_new(t: i32) -> Option<Self> {
        if !(0..=24 * 60).contains(&t) {
            return None;
        }
        Self::try_hm(t as u32 / 60, t as u32 % 60)
    }

    pub fn new(t: u32) -> Self {
        Self::hm(t / 60, t % 60)
    }

    pub fn parse_prefix(input: &str) -> (ParseResult<Time, ()>, &str) {
        if let Some(c) = TIME_HM.captures(input) {
            (convert_hm(&c).into(), rest(c, input))
        } else if let Some(c) = TIME_DEC.captures(input) {
            let h = u32::from_str(c.name("hour").unwrap().as_str()).unwrap();
            let dec = u32::from_str(c.name("dec").unwrap().as_str()).unwrap();
            (Self::try_hm(h, (dec * 60) / 100).into(), rest(c, input))
        } else if let Some(c) = TIME_SHORT.captures(input) {
            (convert_hm(&c).into(), rest(c, input))
        } else if let Some(c) = TIME_H.captures(input) {
            let h = u32::from_str(c.name("hour").unwrap().as_str()).unwrap();
            (Self::try_hm(h, 0).into(), rest(c, input))
        } else {
            (ParseResult::None, input)
        }
    }

    pub fn parse_with_offset<'a, 'b>(
        timeline: &'b Timeline,
        input: &'a str,
    ) -> (ParseResult<Time, ()>, &'a str) {
        let t1 = Time::parse_prefix(input);
        match t1 {
            (ParseResult::None | ParseResult::Incomplete, _) => {
                let (tr, rest) = TimeRelative::parse_relative(input);
                (
                    tr.and_then(|r| timeline.time_now().try_add_relative(r).into()),
                    rest,
                )
            }
            absolute => absolute,
        }
    }

    pub fn check_hm(h: u32, m: u32) -> ParseResult<Time, ()> {
        if h >= 24 || m >= 60 {
            ParseResult::Invalid(())
        } else {
            ParseResult::Valid(Time::hm(h, m))
        }
    }

    pub fn check_hp(h: u32, p: u32) -> ParseResult<Time, ()> {
        if h >= 24 || p >= 100 {
            ParseResult::Invalid(())
        } else {
            let m = (p * 6) / 10;
            ParseResult::Valid(Time::hm(h, m))
        }
    }

    pub fn try_add_relative(self, tr: TimeRelative) -> Option<Self> {
        let v = self.h as i32 * 60 + self.m as i32 + tr.offset_minutes();
        Self::try_new(v)
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

    pub fn round(self, mode: RoundMode, resolution: NonZeroU32) -> Self {
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
            RoundMode::Down => {
                let m = (m / resolution) * resolution.get();
                Self::hm(h, m)
            }
            RoundMode::Up => {
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

fn convert_hm(c: &Captures) -> Option<Time> {
    let h = u32::from_str(c.name("hour").unwrap().as_str()).unwrap();
    let m = u32::from_str(c.name("minute").unwrap().as_str()).unwrap();
    Time::try_hm(h, m)
}

impl serde::Serialize for Time {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Time {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(TimeVisitor)
    }
}

struct TimeVisitor;
impl<'de> serde::de::Visitor<'de> for TimeVisitor {
    type Value = Time;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "Time in format 'hh:mm'")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some(c) = TIME_HM.captures(v) {
            convert_hm(&c).ok_or_else(|| E::custom(format!("Out of range: {}", v)))
        } else {
            Err(E::custom(format!("invalid time: {}", v)))
        }
    }
}

lazy_static::lazy_static! {
    static ref TIME_HM: Regex = Regex::new(r"^(?P<hour>[0-9]{1,2}):(?P<minute>[0-9]{1,2})\b").unwrap();
    static ref TIME_SHORT: Regex = Regex::new(r"^(?P<hour>[0-9]{1,2})(?P<minute>[0-9]{2})\b").unwrap();
    static ref TIME_H: Regex = Regex::new(r"^(?P<hour>[0-9]{1,2})\b").unwrap();
    static ref TIME_DEC: Regex = Regex::new(r"^(?P<hour>[0-9]{1,2})\.(?P<dec>[0-9]{1,2})\b").unwrap();
}

impl From<Time> for chrono::NaiveTime {
    fn from(t: Time) -> Self {
        if t.h == 24 {
            chrono::NaiveTime::from_hms(23, 59, 59)
        } else {
            chrono::NaiveTime::from_hms(t.h(), t.m(), 0)
        }
    }
}

impl From<&Time> for chrono::NaiveTime {
    fn from(t: &Time) -> Self {
        chrono::NaiveTime::from(*t)
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
        write!(f, "{:02}:{:02}", self.h(), self.m())
    }
}

impl Add<TimeRelative> for Time {
    type Output = Time;

    fn add(self, rhs: TimeRelative) -> Self::Output {
        let x = TimeRelative::new(false, self.h, self.m).unwrap() + rhs;
        if x.offset_minutes() < 0 {
            Time::ZERO
        } else {
            Time::try_new(x.offset_minutes()).unwrap()
        }
    }
}

impl Sub for Time {
    type Output = TimeRelative;

    fn sub(self, rhs: Time) -> Self::Output {
        let r = (rhs.h as i32) * 60 + rhs.m as i32;
        let l = (self.h as i32) * 60 + self.m as i32;

        let diff_minutes = l - r;
        TimeRelative::from_minutes(diff_minutes).unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::parsing::time::Time;
    use crate::parsing::time_relative::TimeRelative;

    #[test]
    fn test_sub() {
        assert_eq!(
            Time::hm(24, 0) - Time::hm(0, 0),
            TimeRelative::new(false, 24, 0).unwrap()
        );
        assert_eq!(
            Time::hm(0, 0) - Time::hm(24, 0),
            TimeRelative::new(true, 24, 0).unwrap()
        );
        assert_eq!(
            Time::hm(0, 0) - Time::hm(0, 0),
            TimeRelative::new(false, 0, 0).unwrap()
        );
        assert_eq!(
            Time::hm(10, 0) - Time::hm(8, 30),
            TimeRelative::new(false, 1, 30).unwrap()
        );
    }

    #[test]
    fn test_try_add_relative() {
        let time = Time::hm(1, 0);
        assert_eq!(
            time.try_add_relative(TimeRelative::new(true, 0, 1).unwrap()),
            Some(Time::hm(0, 59))
        );
    }

    #[test]
    fn test_time_add_duration() {
        assert_eq!(
            Time::hm(0, 0) + TimeRelative::from_minutes_sat(30),
            Time::hm(0, 30)
        );

        assert_eq!(
            Time::hm(0, 0) + TimeRelative::from_minutes_sat(600),
            Time::hm(10, 0)
        );
        assert_eq!(
            Time::hm(0, 0) + TimeRelative::from_minutes_sat(630),
            Time::hm(10, 30)
        );
        assert_eq!(
            Time::hm(9, 0) + TimeRelative::from_minutes_sat(120),
            Time::hm(11, 0)
        );
    }

    #[test]
    #[allow(clippy::eq_op)]
    fn test_ord() {
        let t8 = Time::hm(8, 45);
        let t9 = Time::hm(9, 15);
        assert!(t8 < t9);
        assert!(!(t9 < t8));
        assert!(t8 <= t8);
        assert!(t8 >= t8);
        assert!(t9 > t8)
    }
}
