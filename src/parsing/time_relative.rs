use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
    pub fn new(neg: bool, h: u8, m: u8) -> Option<TimeRelative> {
        if h > 12 {
            None
        } else if m >= 60 {
            None
        } else {
            if neg {
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
    }

    pub fn is_negative(&self) -> bool {
        self.h < 0 || self.m < 0
    }

    pub fn offset_hours(&self) -> i32 {
        self.h as i32
    }

    pub fn offset_minutes(&self) -> i32 {
        self.m as i32
    }

    pub fn parse_relaxed(input: &str) -> Option<(TimeRelative, &str)> {
        let (neg, input) = if input.starts_with('+') {
            (false, &input[1..])
        } else if input.starts_with('-') {
            (true, &input[1..])
        } else {
            (false, input)
        };

        Self::parse_body(neg, input)
    }

    pub fn parse_prefix(input: &str) -> Option<(TimeRelative, &str)> {
        let (neg, input) = if input.starts_with('+') {
            (false, &input[1..])
        } else if input.starts_with('-') {
            (true, &input[1..])
        } else {
            return None;
        };

        Self::parse_body(neg, input)
    }

    fn parse_body(neg: bool, input: &str) -> Option<(TimeRelative, &str)> {
        #[derive(Debug)]
        enum Unit {
            H,
            M,
        }
        let (first_num, unit, input) = {
            let (head, tail) = str_split_at(input, |c: char| !c.is_ascii_digit());
            let num: u8 = u8::from_str(head).ok()?;
            if tail.starts_with('h') {
                (num, Unit::H, &tail[1..])
            } else if tail.starts_with('m') {
                (num, Unit::M, &tail[1..])
            } else {
                (num, Unit::M, tail)
            }
        };

        if let Unit::H = unit {
            let (head, tail) = str_split_at(input, |c: char| !c.is_ascii_digit());
            let (minutes, tail) = if head.is_empty() {
                if tail.is_empty() || tail.starts_with(|c: char| c.is_ascii_whitespace()) {
                    (0, tail)
                } else {
                    return None;
                }
            } else {
                let num = u8::from_str(head).ok()?;
                if tail.starts_with('m') {
                    (num, &tail[1..])
                } else if tail.is_empty() || tail.starts_with(|c: char| c.is_ascii_whitespace()) {
                    (num, tail)
                } else {
                    return None;
                }
            };
            check_h_m(neg, first_num, minutes, tail)
        } else {
            let h = first_num / 60;
            let m = first_num % 60;
            check_h_m(neg, h, m, input)
        }
    }
}

fn str_split_at<'a, P: FnMut(char) -> bool>(s: &str, p: P) -> (&str, &str) {
    if let Some(start) = s.find(p) {
        (&s[0..start], &s[start..])
    } else {
        (s, "")
    }
}

fn check_h_m(neg: bool, h: u8, m: u8, tail: &str) -> Option<(TimeRelative, &str)> {
    TimeRelative::new(neg, h, m).map(|tr| (tr, tail))
}

#[cfg(test)]
mod test {
    use crate::parsing::time_relative::{str_split_at, TimeRelative};

    #[test]
    fn test_str_split_at() {
        assert_eq!(str_split_at("", |c| c == 'a'), ("", ""));
        assert_eq!(str_split_at("abc", |c| c == 'd'), ("abc", ""));
        assert_eq!(str_split_at("abc", |c| c == 'c'), ("ab", "c"));
        assert_eq!(str_split_at("abc", |c| c == 'a'), ("", "abc"));
    }

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
        ])
        .unwrap();
        assert_no_parse(&["+24h", "h", "++1h", "-+1h", "1hh"]).unwrap();
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
            if let Some(r) = TimeRelative::parse_relaxed(input) {
                return Err(format!(
                    "Did not expect that '{}' parses into {} with tail '{}'",
                    input, r.0, r.1
                ));
            }
        }
        Ok(())
    }

    fn assert_parse(v: &[(&str, &str, &str)]) -> Result<(), String> {
        for (input, expected, rest) in v {
            let (parsed, tail) = TimeRelative::parse_relaxed(input)
                .ok_or(format!("Could not parse {} into {}", input, expected))?;
            let result = parsed.to_string();
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
