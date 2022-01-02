use chrono::{Datelike, Weekday};
use serde::{Deserializer, Serializer};
use std::fmt::{Display, Formatter};
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
        self.next(WeekDayForwarder)
    }

    pub fn prev_day(&self) -> Day {
        self.prev(SimpleDayForwarder)
    }

    pub fn next(&self, forwarder: impl DayForwarder) -> Day {
        forwarder.next_day(*self)
    }

    pub fn prev(&self, forwarder: impl DayForwarder) -> Day {
        forwarder.prev_day(*self)
    }

    pub fn ymd(year: i32, month: u32, day: u32) -> Day {
        Day {
            date: chrono::NaiveDate::from_ymd(year, month, day),
        }
    }
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

impl serde::Serialize for Day {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
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
        if let Some((year, month_day)) = v.split_once('-') {
            if let Some((month, day)) = month_day.split_once('-') {
                return match (
                    i32::from_str(year),
                    u32::from_str(month),
                    u32::from_str(day),
                ) {
                    (Ok(year), Ok(month), Ok(day)) => {
                        let date = chrono::NaiveDate::from_ymd_opt(year, month, day)
                            .ok_or_else(|| E::custom("bad date"))?;
                        Ok(Day { date })
                    }
                    _ => Err(E::custom("invalid date")),
                };
            }
        }

        Err(E::custom(format!("Invalid date: {}", v)))
    }
}

pub trait DayForwarder {
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

pub struct SimpleDayForwarder;

impl DayForwarder for SimpleDayForwarder {
    fn is_valid(&self, _: Day) -> bool {
        true
    }
}

pub struct WeekDayForwarder;

impl DayForwarder for WeekDayForwarder {
    fn is_valid(&self, day: Day) -> bool {
        let weekday = day.date.weekday();
        weekday != Weekday::Sat && weekday != Weekday::Sun
    }
}

#[cfg(test)]
mod test {
    use crate::data::day::Day;
    use crate::data::WeekDayForwarder;

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
        let prev_friday = start_monday.prev(WeekDayForwarder);
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
}
