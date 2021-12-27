use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::time_relative::TimeRelative;
use std::str::FromStr;

pub fn parse_absolute(input: &str) -> ParseResult<Time, ()> {
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
            return Time::check_hm(h, m);
        }
    } else if let Some((h, p)) = input.split_once(&[',', '.'][..]) {
        if p.starts_with('+') {
            return ParseResult::Invalid(());
        }
        if p.is_empty() {
            return ParseResult::Incomplete;
        }
        if let (Ok(h), Ok(p)) = (u32::from_str(h), u32::from_str(p)) {
            return Time::check_hp(h, p);
        }
    } else if let Ok(t) = u32::from_str(input) {
        if t < 24 {
            return Time::check_hm(t, 0);
        } else if t > 100 && t <= 2359 {
            return Time::check_hm(t / 100, t % 100);
        }
    }

    ParseResult::Invalid(())
}

pub fn parse_input_rel(now: Time, text: &str, negate: bool) -> ParseResult<Time, ()> {
    if text.eq_ignore_ascii_case("now") || text.eq_ignore_ascii_case("n") {
        ParseResult::Valid(now)
    } else {
        match parse_absolute(text) {
            r @ (ParseResult::None | ParseResult::Incomplete | ParseResult::Valid(_)) => r,
            ParseResult::Invalid(()) => {
                let (r, rest) = TimeRelative::parse_prefix(text);

                let ts =
                    r.and_then(
                        |rel| match now.try_add_relative(if negate { -rel } else { rel }) {
                            Some(ts) => ParseResult::Valid(ts),
                            None => ParseResult::Invalid(()),
                        },
                    );

                match ts {
                    ParseResult::Valid(ts) => {
                        if !rest.trim().is_empty() {
                            ParseResult::Invalid(())
                        } else {
                            ParseResult::Valid(ts)
                        }
                    }
                    r => r,
                }
            }
        }
    }
}

pub fn parse_input(now: Time, text: &str) -> ParseResult<Time, ()> {
    parse_input_rel(now, text, false)
}
