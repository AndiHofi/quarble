use crate::parsing::parse_result::ParseResult;
use crate::util::Timeline;
use chrono::{Datelike, Duration, Weekday};
use regex::Regex;
use serde::{Deserializer, Serializer};
use std::fmt::{Display, Formatter};
use std::ops::{Add, Sub};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Day {
    date: chrono::NaiveDate,
}

impl Day {
    pub fn today() -> Day {
        let secs = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        let date = chrono::NaiveDateTime::from_timestamp(secs, 0).date();

        Day { date }
    }

    pub fn next_work_day(&self) -> Day {
        self.next(&WeekDayForwarder)
    }

    pub fn next_day(&self) -> Day {
        self.next(&SimpleDayForwarder)
    }

    pub fn prev_day(&self) -> Day {
        self.prev(&SimpleDayForwarder)
    }

    pub fn next(&self, forwarder: &impl DayForwarder) -> Day {
        forwarder.next_day(*self)
    }

    pub fn prev(&self, forwarder: &impl DayForwarder) -> Day {
        forwarder.prev_day(*self)
    }

    pub fn ymd(year: i32, month: u32, day: u32) -> Day {
        Day {
            date: chrono::NaiveDate::from_ymd(year, month, day),
        }
    }

    pub fn parse(input: &str) -> Result<Day, String> {
        parse_day(input)
    }

    pub fn iter<F: DayForwarder>(self, forwarder: F) -> DayIter<F> {
        DayIter {
            day: self,
            forwarder,
        }
    }

    pub fn day_of_week(self) -> chrono::Weekday {
        self.date.weekday()
    }

    pub fn parse_day_relative(timeline: &Timeline, input: &str) -> ParseResult<Day, ()> {
        if let Some(c) = RELATIVE_DAY.captures(input) {
            let sign = c.name("sign").unwrap().as_str() == "+";
            let days = i32::from_str(c.name("days").unwrap().as_str()).unwrap();
            let mut value = timeline.today();
            if sign {
                for _ in 0..days {
                    value = value.next(&SimpleDayForwarder);
                }
            } else {
                for _ in 0..days {
                    value = value.prev(&SimpleDayForwarder);
                }
            }
            ParseResult::Valid(value)
        } else {
            parse_day(input).map_err(|_| ()).into()
        }
    }

    pub fn add_with_forwarder(self, amount: i64, forwarder: &dyn DayForwarder) -> Day {
        let mut result = self;
        let mut remain = amount;
        match amount {
            _ if amount < 0 => {
                while remain < 0 {
                    result = forwarder.prev_day(result);
                    remain += 1;
                }
            }
            _ if amount > 0 => {
                while remain > 0 {
                    result = forwarder.next_day(result);
                    remain -= 1;
                }
            }
            _ => (),
        }

        result
    }
}

fn parse_day(input: &str) -> Result<Day, String> {
    if let Some((year, month_day)) = input.split_once('-') {
        if let Some((month, day)) = month_day.split_once('-') {
            return match (
                i32::from_str(year),
                u32::from_str(month),
                u32::from_str(day),
            ) {
                (Ok(year), Ok(month), Ok(day)) => {
                    let date = chrono::NaiveDate::from_ymd_opt(year, month, day)
                        .ok_or_else(|| "bad date".to_string())?;
                    Ok(Day { date })
                }
                _ => Err("invalid date".to_string()),
            };
        }
    }

    Err(format!("Invalid date: {}", input))
}

lazy_static::lazy_static! {
    static ref RELATIVE_DAY: Regex = Regex::new(r"^(?P<sign>\+|-)(?P<days>[0-9]{1,2})\b").unwrap();
}

impl Default for Day {
    fn default() -> Self {
        Day::today()
    }
}

impl Display for Day {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:04}-{:02}-{:02}",
            self.date.year(),
            self.date.month(),
            self.date.day()
        )
    }
}

impl From<chrono::NaiveDate> for Day {
    fn from(date: chrono::NaiveDate) -> Self {
        Day { date }
    }
}

impl From<Day> for chrono::NaiveDate {
    fn from(d: Day) -> Self {
        d.date
    }
}

impl serde::Serialize for Day {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl Add<i64> for Day {
    type Output = Self;
    fn add(self, rhs: i64) -> Self::Output {
        Day {
            date: self.date.add(Duration::days(rhs)),
        }
    }
}

impl Sub<i64> for Day {
    type Output = Self;
    fn sub(self, rhs: i64) -> Self::Output {
        self + (-rhs)
    }
}

impl<'de> serde::Deserialize<'de> for Day {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DayVisitor)
    }
}

struct DayVisitor;

impl<'de> serde::de::Visitor<'de> for DayVisitor {
    type Value = Day;

    fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
        formatter.write_str("ISO date in the format YYYY-MM-DD")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        parse_day(v).map_err(E::custom)
    }
}

pub trait DayForwarder: Send + Sync + std::fmt::Debug {
    fn next_day(&self, day: Day) -> Day {
        day.date
            .iter_days()
            .skip(1)
            .map(|d| Day { date: d })
            .find(|d| self.is_valid(*d))
            .unwrap()
    }

    fn prev_day(&self, day: Day) -> Day {
        let mut day = Day {
            date: day.date.pred(),
        };
        while !self.is_valid(day) {
            day = Day {
                date: day.date.pred(),
            }
        }
        day
    }

    fn is_valid(&self, day: Day) -> bool;
}

#[derive(Copy, Clone, Debug)]
pub struct SimpleDayForwarder;

impl DayForwarder for SimpleDayForwarder {
    fn is_valid(&self, _: Day) -> bool {
        true
    }
}

#[derive(Copy, Clone, Debug)]
pub struct WeekDayForwarder;

impl DayForwarder for WeekDayForwarder {
    fn is_valid(&self, day: Day) -> bool {
        let weekday = day.date.weekday();
        weekday != Weekday::Sat && weekday != Weekday::Sun
    }
}

pub struct DayIter<Forwarder> {
    day: Day,
    forwarder: Forwarder,
}

impl<F: DayForwarder> Iterator for DayIter<F> {
    type Item = Day;
    fn next(&mut self) -> Option<Self::Item> {
        self.day = self.day.next(&self.forwarder);
        Some(self.day)
    }
}

#[cfg(test)]
mod test {
    use crate::data::day::Day;
    use crate::data::WeekDayForwarder;
    use crate::util::{DefaultTimeline, TimelineProvider};

    #[test]
    fn day_serde_json() {
        let day = Day::ymd(2021, 11, 28);
        let as_str = serde_json::to_string(&day).unwrap();
        assert_eq!(as_str, "\"2021-11-28\"");
        let from_str: Day = serde_json::from_str(&as_str).unwrap();
        assert_eq!(from_str, day);
    }

    #[test]
    fn next_work_day() {
        let start_friday = Day::ymd(2021, 11, 26);
        let next = start_friday.next_work_day();
        assert_eq!(next, Day::ymd(2021, 11, 29));
    }

    #[test]
    fn prev_work_day() {
        let start_monday = Day::ymd(2021, 11, 29);
        let prev_friday = start_monday.prev(&WeekDayForwarder);
        assert_eq!(prev_friday, Day::ymd(2021, 11, 26));
    }

    #[test]
    fn prev_day() {
        let start = Day::ymd(2021, 11, 29);
        let prev = start.prev_day();
        assert_eq!(prev, Day::ymd(2021, 11, 28));
        assert_eq!(prev.prev_day(), Day::ymd(2021, 11, 27));
        assert_eq!(Day::ymd(2021, 1, 1).prev_day(), Day::ymd(2020, 12, 31));
    }

    #[test]
    fn test_forwarder_add() {
        eprintln!(
            "{}",
            DefaultTimeline
                .today()
                .prev(&WeekDayForwarder)
                .day_of_week()
        );

        let start = Day::ymd(2022, 1, 17);
        let same = start.add_with_forwarder(0, &WeekDayForwarder);
        assert_eq!(same, start);

        let next = start.add_with_forwarder(6, &WeekDayForwarder);
        assert_eq!(next, Day::ymd(2022, 1, 25));

        let prev_friday = start.add_with_forwarder(-1, &WeekDayForwarder);
        dbg!(prev_friday.day_of_week());
        assert_eq!(prev_friday, Day::ymd(2022, 1, 14));

        let prev = start.add_with_forwarder(-6, &WeekDayForwarder);
        eprintln!("{}", prev.day_of_week());
        assert_eq!(prev, Day::ymd(2022, 1, 7));
    }
}
