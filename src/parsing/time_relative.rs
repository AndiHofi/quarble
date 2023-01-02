use crate::parsing::parse_result::ParseResult;
use crate::parsing::round_mode::RoundMode;
use crate::parsing::time::Time;
use crate::parsing::time_relative::parse::{
    parse_duration, parse_duration_relaxed, parse_time_relative,
};
use std::fmt::{Display, Formatter};
use std::num::NonZeroU32;
use std::ops::{Add, AddAssign, Neg, Sub};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TimeRelative {
    h: i8,
    m: i8,
}

impl Display for TimeRelative {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (pre, h, m) = if self.h < 0 || self.m < 0 {
            ("-", -self.h, -self.m)
        } else {
            ("+", self.h, self.m)
        };
        if h == 0 && m == 0 {
            return f.write_str("0");
        }

        f.write_str(pre)?;
        if h != 0 {
            write!(f, "{}h", h)?;
        }

        if m != 0 {
            write!(f, "{}m", m)?;
        }
        Ok(())
    }
}

impl TimeRelative {
    pub const ZERO: TimeRelative = TimeRelative::from_minutes_sat(0);

    pub const fn new(neg: bool, h: u8, m: u8) -> Option<TimeRelative> {
        if !(h == 24 && m == 0 || h < 24 && m < 60) {
            None
        } else if neg {
            Some(TimeRelative {
                h: 0 - (h as i8),
                m: 0 - (m as i8),
            })
        } else {
            Some(TimeRelative {
                h: h as i8,
                m: m as i8,
            })
        }
    }

    const fn new_unsafe(neg: bool, h: u8, m: u8) -> TimeRelative {
        if !(h == 24 && m == 0 || h < 24 && m < 60) {
            panic!("Invalid TimeRelative");
        } else if neg {
            TimeRelative {
                h: 0 - (h as i8),
                m: 0 - (m as i8),
            }
        } else {
            TimeRelative {
                h: h as i8,
                m: m as i8,
            }
        }
    }

    pub fn from_minutes(minutes: i32) -> Option<TimeRelative> {
        let negative = minutes < 0;
        let minutes = minutes.abs();
        if minutes > 60 * 24 {
            return None;
        }
        Self::new(negative, (minutes / 60) as u8, (minutes % 60) as u8)
    }

    pub const fn from_minutes_sat(mut minutes: i32) -> TimeRelative {
        let negative = minutes < 0;
        if minutes < 0 {
            minutes = -minutes;
        };
        if minutes > 24 * 60 {
            minutes = 24 * 60;
        }
        Self::new_unsafe(negative, (minutes / 60) as u8, (minutes % 60) as u8)
    }

    pub fn is_negative(&self) -> bool {
        self.h < 0 || self.m < 0
    }

    pub fn offset_minutes(&self) -> i32 {
        self.h as i32 * 60 + self.m as i32
    }

    pub fn parse_relaxed(input: &str) -> (ParseResult<TimeRelative, ()>, &str) {
        parse_duration_relaxed(input)
    }

    pub fn parse_prefix(input: &str) -> (ParseResult<TimeRelative, ()>, &str) {
        parse_time_relative(input)
    }

    pub fn parse_relative(input: &str) -> (ParseResult<TimeRelative, ()>, &str) {
        parse_time_relative(input)
    }

    pub fn parse_duration(input: &str) -> (ParseResult<TimeRelative, ()>, &str) {
        parse_duration(input)
    }

    pub fn abs(self) -> Self {
        Self {
            h: self.h.abs(),
            m: self.m.abs(),
        }
    }

    pub fn round(self, mode: RoundMode, resolution: NonZeroU32) -> Self {
        let rounded = Time::new(self.offset_minutes().abs() as u32).round(mode, resolution);
        TimeRelative::new(self.is_negative(), rounded.h() as u8, rounded.m() as u8).unwrap()
    }
}

impl AddAssign for TimeRelative {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Add for TimeRelative {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        TimeRelative::from_minutes_sat(self.offset_minutes() + rhs.offset_minutes())
    }
}

impl Sub for TimeRelative {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        TimeRelative::from_minutes_sat(self.offset_minutes() - rhs.offset_minutes())
    }
}

impl Neg for TimeRelative {
    type Output = TimeRelative;
    fn neg(self) -> Self::Output {
        TimeRelative {
            h: -self.h,
            m: -self.m,
        }
    }
}

mod parse {
    use crate::parsing::parse_result::ParseResult;
    use crate::parsing::rest;
    use crate::parsing::time_relative::TimeRelative;
    use lazy_static::lazy_static;
    use regex::{Captures, Regex};
    use std::str::FromStr;

    lazy_static! {
        static ref POSITIVE_DURATION_HOUR: Regex =
            regex::Regex::new(r"^(?P<hour>[0-9]{1,2})h((?P<minute>[0-9]{1,2})(m)?)?\b").unwrap();
        static ref POSITIVE_DURATION_MINUTE: Regex =
            Regex::new(r"^(?P<minute>[0-9]{1,3})m\b").unwrap();
        static ref RELATIVE_TIME_MIN: Regex =
            Regex::new(r"^(?P<sign>\+|\-)(?P<minute>[0-9]{1,3})(?:m)?\b").unwrap();
        static ref RELATIVE_TIME_HOUR: Regex =
            Regex::new(r"^(?P<sign>\+|\-)(?P<hour>[0-9]{1,2})h((?P<minute>[0-9]{1,2})m)?\b")
                .unwrap();
        static ref NOW: regex::Regex = Regex::new(r"^(?:n|now)\b").unwrap();
        static ref JUST_MINUTES: Regex = Regex::new(r"^(?P<minute>[0-9]{1,3})\b").unwrap();
    }

    pub(super) fn parse_duration(input: &str) -> (ParseResult<TimeRelative, ()>, &str) {
        if let Some(c) = POSITIVE_DURATION_HOUR.captures(input) {
            (take_hm(false, &c), rest(c, input))
        } else if let Some(c) = POSITIVE_DURATION_MINUTE.captures(input) {
            (take_minutes(false, &c), rest(c, input))
        } else {
            (ParseResult::None, input)
        }
    }

    pub(super) fn parse_time_relative(input: &str) -> (ParseResult<TimeRelative, ()>, &str) {
        if let Some(c) = RELATIVE_TIME_HOUR.captures(input) {
            (take_hm(take_negative(&c), &c), rest(c, input))
        } else if let Some(c) = RELATIVE_TIME_MIN.captures(input) {
            (take_minutes(take_negative(&c), &c), rest(c, input))
        } else if let Some(c) = NOW.captures(input) {
            (
                ParseResult::Valid(TimeRelative::new(false, 0, 0).unwrap()),
                rest(c, input),
            )
        } else {
            (ParseResult::None, input)
        }
    }

    pub(super) fn parse_duration_relaxed(input: &str) -> (ParseResult<TimeRelative, ()>, &str) {
        match parse_time_relative(input) {
            (ParseResult::None, _) => match parse_duration(input) {
                (ParseResult::None, _) => parse_minutes(input),
                r => r,
            },
            r => r,
        }
    }

    fn parse_minutes(input: &str) -> (ParseResult<TimeRelative, ()>, &str) {
        if let Some(c) = JUST_MINUTES.captures(input) {
            (take_minutes(false, &c), rest(c, input))
        } else {
            (ParseResult::None, input)
        }
    }

    fn take_negative(c: &Captures) -> bool {
        let sign = c.name("sign").unwrap().as_str();
        sign == "-"
    }

    fn take_hm(negative: bool, c: &Captures) -> ParseResult<TimeRelative, ()> {
        let h = u8::from_str(c.name("hour").unwrap().as_str()).unwrap();
        let m = c
            .name("minute")
            .map(|m| u8::from_str(m.as_str()).unwrap())
            .unwrap_or(0);
        TimeRelative::new(negative, h, m).into()
    }

    fn take_minutes<'a>(negative: bool, c: &'a Captures<'a>) -> ParseResult<TimeRelative, ()> {
        let m = c
            .name("minute")
            .map(|m| u16::from_str(m.as_str()).unwrap())
            .unwrap_or(0);
        if m > 24 * 60 {
            return ParseResult::Invalid(());
        }
        let h = (m / 60) as u8;
        let m = (m % 60) as u8;
        TimeRelative::new(negative, h, m).into()
    }

    #[cfg(test)]
    mod test {
        use crate::parsing::parse_result::ParseResult;
        use crate::parsing::time_relative::parse::{parse_duration, parse_time_relative};
        use crate::parsing::time_relative::TimeRelative;

        fn valid(h: i8, m: i8) -> ParseResult<TimeRelative, ()> {
            let negative = h < 0 || m < 0;
            let h = h.abs() as u8;
            let m = m.abs() as u8;

            ParseResult::Valid(TimeRelative::new(negative, h, m).unwrap())
        }

        #[test]
        fn test_parse_time_relative() {
            assert_eq!(parse_time_relative("+10h"), (valid(10, 0), ""));
            assert_eq!(parse_time_relative("-10h"), (valid(-10, 0), ""));
            assert_eq!(parse_time_relative("+1h15m"), (valid(1, 15), ""));
            assert_eq!(parse_time_relative("-1h15m"), (valid(-1, -15), ""));
            assert_eq!(parse_time_relative("+90m"), (valid(1, 30), ""));
            assert_eq!(parse_time_relative("-90m"), (valid(-1, -30), ""));
            assert_eq!(parse_time_relative("+600m"), (valid(10, 0), ""));
            assert_eq!(parse_time_relative("-600m"), (valid(-10, 0), ""));
            assert_eq!(parse_time_relative("+25h"), (ParseResult::Invalid(()), ""));
            assert_eq!(parse_time_relative("-25h"), (ParseResult::Invalid(()), ""));
            assert_eq!(
                parse_time_relative("+2h90m"),
                (ParseResult::Invalid(()), "")
            );
            assert_eq!(
                parse_time_relative("-2h90m"),
                (ParseResult::Invalid(()), "")
            );
            assert_eq!(parse_time_relative("+999m"), (valid(16, 39), ""));
            assert_eq!(parse_time_relative("-999m"), (valid(-16, -39), ""));
            assert_eq!(parse_time_relative("+90"), (valid(1, 30), ""));
            assert_eq!(parse_time_relative("-90"), (valid(-1, -30), ""));
            assert_eq!(parse_time_relative("++90"), (ParseResult::None, "++90"));
            assert_eq!(parse_time_relative("--90"), (ParseResult::None, "--90"));
            assert_eq!(parse_time_relative("+hm"), (ParseResult::None, "+hm"));
            assert_eq!(parse_time_relative("++90m"), (ParseResult::None, "++90m"));

            assert_eq!(parse_time_relative("-1h 1h"), (valid(-1, 0), " 1h"));
        }

        #[test]
        fn test_parse_duration() {
            assert_eq!(parse_duration("10h"), (valid(10, 0), ""));
            assert_eq!(parse_duration("1h15m"), (valid(1, 15), ""));
            assert_eq!(parse_duration("90m"), (valid(1, 30), ""));
            assert_eq!(parse_duration("600m"), (valid(10, 0), ""));
            assert_eq!(parse_duration("25h"), (ParseResult::Invalid(()), ""));
            assert_eq!(parse_duration("2h90m"), (ParseResult::Invalid(()), ""));
            assert_eq!(parse_duration("999m"), (valid(16, 39), ""));
            assert_eq!(parse_duration("90"), (ParseResult::None, "90"));
            assert_eq!(parse_duration("hm"), (ParseResult::None, "hm"));
            assert_eq!(parse_duration("+90m"), (ParseResult::None, "+90m"));

            assert_eq!(parse_duration("1h 1h"), (valid(1, 0), " 1h"));
        }
    }
}

#[cfg(test)]
mod test {
    use crate::parsing::parse_result::ParseResult;
    use crate::parsing::time_relative::TimeRelative;

    #[test]
    fn parse_simple_num() {
        assert_parse(&[
            ("0", "0", ""),
            ("-0", "0", ""),
            ("15", "+15m", ""),
            ("-15", "-15m", ""),
            ("90", "+1h30m", ""),
            ("-255", "-4h15m", ""),
            ("15 abc", "+15m", " abc"),
        ])
        .unwrap();
    }

    #[test]
    fn parse_m() {
        assert_parse(&[
            ("0m", "0", ""),
            ("-0m", "0", ""),
            ("15m", "+15m", ""),
            ("-15m", "-15m", ""),
            ("90m", "+1h30m", ""),
            ("-255m", "-4h15m", ""),
            ("-120m", "-2h", ""),
            ("+120m", "+2h", ""),
            ("120m", "+2h", ""),
            ("120", "+2h", ""),
            ("15m abc", "+15m", " abc"),
        ])
        .unwrap();
    }

    #[test]
    fn parse_h() {
        assert_parse(&[
            ("0h", "0", ""),
            ("-0h", "0", ""),
            ("1h", "+1h", ""),
            ("-12h", "-12h", ""),
            ("+12h", "+12h", ""),
            ("+1h h", "+1h", " h"),
            ("+24h h", "+24h", " h"),
        ])
        .unwrap();
        assert_no_parse(&["+25h", "h", "++1h", "-+1h", "+h"]).unwrap();
    }

    #[test]
    fn parse_h_m() {
        assert_parse(&[
            ("0h0m", "0", ""),
            ("-0h0m", "0", ""),
            ("12h59m", "+12h59m", ""),
            ("-0h1m", "-1m", ""),
        ])
        .unwrap();
    }

    fn assert_no_parse(v: &[&str]) -> Result<(), String> {
        for input in v {
            if let (ParseResult::Valid(r), tail) = TimeRelative::parse_relaxed(input) {
                return Err(format!(
                    "Did not expect that '{}' parses into {} with tail '{}'",
                    input, r, tail
                ));
            }
        }
        Ok(())
    }

    fn assert_parse(v: &[(&str, &str, &str)]) -> Result<(), String> {
        for (input, expected, rest) in v {
            let (parsed, tail) = TimeRelative::parse_relaxed(input);
            parsed
                .as_ref()
                .get()
                .ok_or(format!("Could not parse {} into {}", input, expected))?;
            let result = parsed.get().unwrap().to_string();
            if &result != expected {
                return Err(format!(
                    "Parsed {} into {}, but expected {}",
                    input, result, expected
                ));
            }
            if tail != *rest {
                return Err(format!(
                    "Parsed {} got rest '{}', but expected rest '{}'",
                    input, tail, rest
                ));
            }
        }

        Ok(())
    }
}
