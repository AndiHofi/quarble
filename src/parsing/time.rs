use std::fmt::{Display, Formatter};
use std::num::NonZeroU32;
use std::str::FromStr;

use crate::parsing::parse_result::ParseResult;
use crate::parsing::round_mode::RoundMode;
use chrono::Timelike;

use crate::parsing::time_relative::TimeRelative;

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

    pub fn try_new(t: u32) -> Option<Self> {
        Self::try_hm(t / 60, t % 60)
    }

    pub fn new(t: u32) -> Self {
        Self::hm(t / 60, t % 60)
    }

    pub fn parse(input: &str) -> ParseResult<Time, ()> {
        if input.is_empty() {
            return ParseResult::Incomplete;
        } else if input.len() > 5 || input.starts_with('+') {
            return ParseResult::Invalid(());
        } else if let Some((h, m)) = input.split_once(':') {
            if m.starts_with('+') {
                return ParseResult::Invalid(());
            }
            if m.is_empty() {
                return ParseResult::Incomplete;
            }
            if let (Ok(h), Ok(m)) = (u32::from_str(h), u32::from_str(m)) {
                return Self::check_hm(h, m);
            }
        } else if let Some((h, p)) = input.split_once(&[',', '.'][..]) {
            if p.starts_with('+') {
                return ParseResult::Invalid(());
            }
            if p.is_empty() {
                return ParseResult::Incomplete;
            }
            if let (Ok(h), Ok(p)) = (u32::from_str(h), u32::from_str(p)) {
                return Self::check_hp(h, p);
            }
        } else if let Ok(t) = u32::from_str(input) {
            if t < 24 {
                return Self::check_hm(t, 0);
            } else if t > 100 && t <= 2359 {
                return Self::check_hm(t / 100, t % 100);
            }
        }

        ParseResult::Invalid(())
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
        let mut h = self.h as i32 + tr.offset_hours();
        let mut m = self.m as i32 + tr.offset_minutes();
        if m < 0 {
            m += 60;
            h -= 1;
        } else if m >= 60 {
            m -= 60;
            h += 1;
        }
        if (0..24).contains(&h) && (0..60).contains(&m) {
            Some(Time::hm(h as u32, m as u32))
        } else {
            None
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
        write!(f, "{}:{:02}", self.h(), self.m())
    }
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
