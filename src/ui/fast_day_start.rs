use chrono::{SubsecRound, Timelike};
use iced_wgpu::TextInput;
use iced_winit::widget::{text_input, Column, Row, Space, Text};

use crate::conf::Settings;
use crate::data::{Action, DayStart, Location, TimedAction, WorkDay};
use crate::parsing::time::{Time, TimeLimit, TimeResult};
use crate::parsing::time_relative::TimeRelative;
use crate::ui::{style, MainView, Message, QElement};
use crate::util;

#[derive(Clone, Debug)]
pub enum FastDayStartMessage {
    TextChanged(String),
}

pub(super) struct FastDayStart {
    text: String,
    text_state: text_input::State,
    value: Option<DayStart>,
    message: String,
    limits: Vec<TimeLimit>,
}

impl FastDayStart {
    pub fn for_work_day(work_day: Option<&WorkDay>) -> Box<Self> {
        let (message, limits) = if let Some(work_day) = work_day {
            let mut actions = work_day.actions().to_vec();
            actions.sort();
            (start_day_message(&actions), valid_time_limits_for_day_start(&actions))
        } else {
            ("Start working day".to_string(), Vec::default())
        };
        Box::new(FastDayStart {
            text: String::new(),
            text_state: text_input::State::focused(),
            value: Some(DayStart {
                location: Location::Office,
                ts: util::time_now(),
            }),
            message,
            limits,
        })
    }
    fn update_text(&mut self, new_value: String) -> Option<Message> {
        self.text = new_value;
        self.value = parse_value(&self.text);
        None
    }
}

fn start_day_message(actions: &[Action]) -> String {
    match min_max_booked(actions) {
        (None, None) => "Start working day".to_string(),
        (Some(start), None) | (None, Some(start)) => format!("First action on {}", start),
        (Some(start), Some(end)) => format!("Already booked from {} to {}", start, end),
    }
}

fn min_max_booked(actions: &[Action]) -> (Option<Time>, Option<Time>) {
    match actions {
        [] => (None, None),
        [first] => {
            let (s, e) = first.times();
            (Some(s), e)
        }
        [first, .., last] => {
            let (s, _) = first.times();
            let (e1, e2) = last.times();
            if e2.is_some() {
                (Some(s), e2)
            } else {
                (Some(s), Some(e1))
            }
        }
    }
}

fn valid_time_limits_for_day_start(actions: &[Action]) -> Vec<TimeLimit> {
    let mut result = Vec::new();
    let mut current_limit = TimeLimit::default();
    for action in actions {
        let (min, max) = action.times();
        let (f, s) = if let Some(max) = max {
            let sep = TimeLimit::simple(min, max);
            current_limit.split(sep)
        } else {
            current_limit.split_at(min)
        };
        match (f, s) {
            (TimeLimit::EMPTY, TimeLimit::EMPTY) => (),
            (TimeLimit::EMPTY, s) => current_limit = s,
            (f, TimeLimit::EMPTY) => current_limit = f,
            (f, s) => {
                result.push(f);
                current_limit = s;
            }
        }
    }

    result.push(current_limit);

    result
}

impl MainView for FastDayStart {
    fn new() -> Box<Self> {
        Box::new(FastDayStart {
            text: String::new(),
            text_state: text_input::State::focused(),
            value: Some(DayStart {
                location: Location::Office,
                ts: util::time_now(),
            }),
            message: "Start working day".to_string(),
            limits: Vec::default(),
        })
    }

    fn view<'a>(&'a mut self, _settings: &Settings) -> QElement<'a> {
        let loc_str = self
            .value
            .as_ref()
            .map(|e| e.location.to_string())
            .unwrap_or("Invalid input".to_string());

        let time_str = self
            .value
            .as_ref()
            .map(|e| e.ts.to_string())
            .unwrap_or(String::new());

        Column::with_children(vec![
            Text::new(format!("Day start: [h|o] [+|-]hours or minute: {}", &self.message)).into(),
            Space::with_width(style::SPACE).into(),
            TextInput::new(&mut self.text_state, "now", &self.text, move |input| {
                on_input_change(input)
            })
            .on_submit(on_submit_message(self.value.as_ref()))
            .into(),
            Space::with_width(style::SPACE).into(),
            Row::with_children(vec![
                Text::new(loc_str).into(),
                Space::with_width(style::SPACE).into(),
                Text::new(time_str).into(),
            ])
            .into(),
        ])
        .into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::FDS(msg) => match msg {
                FastDayStartMessage::TextChanged(new_value) => self.update_text(new_value),
            },
            Message::StoreSuccess => Some(Message::Exit),
            _ => None,
        }
    }
}

fn parse_value(text: &str) -> Option<DayStart> {
    let text = text.trim();
    let (location, text) = if text.starts_with(&['h', 'H'][..]) {
        (Location::Home, (&text[1..]).trim())
    } else if text.starts_with(&['o', 'O'][..]) {
        (Location::Office, (&text[1..]).trim())
    } else {
        (Location::Office, text)
    };

    if let TimeResult::Valid(ts) = TimeLimit::default().is_valid(text) {
        Some(DayStart {
            location,
            ts: ts.into(),
        })
    } else if let Some((tr, rest)) = TimeRelative::parse_prefix(text) {
        let now: Time = util::time_now().into();
        let ts = now.try_add_relative(tr)?;
        if rest.trim().is_empty() {
            Some(DayStart {
                location,
                ts: ts.into(),
            })
        } else {
            None
        }
    } else if text.eq_ignore_ascii_case("now") || text.eq_ignore_ascii_case("n") {
        Some(DayStart {
            location,
            ts: util::time_now().with_second(0).unwrap().trunc_subsecs(0),
        })
    } else {
        None
    }
}

fn on_input_change(text: String) -> Message {
    Message::FDS(FastDayStartMessage::TextChanged(text))
}

fn on_submit_message(value: Option<&DayStart>) -> Message {
    if let Some(v) = value {
        Message::StoreAction(Action::DayStart(v.clone()))
    } else {
        Message::Update
    }
}

#[cfg(test)]
mod test {
    use chrono::Timelike;

    use crate::parsing::time::Time;
    use crate::ui::fast_day_start::parse_value;
    use crate::util;

    #[test]
    fn test_parse_value() {
        let c_time = util::time_now();
        eprintln!("{:?}, {}, {}", c_time, c_time.hour(), c_time.minute());
        let time: Time = c_time.into();
        eprintln!("{}", time);
        eprintln!("{}", Time::hm(1, 9));
        p(&[
            "h12", "h12h", "h12m", "h+1h", "h-1m", "h-15", "-15", "o +1h15m", "h+0m ", "h+1",
        ])
    }

    fn p(i: &[&str]) {
        for input in i {
            eprintln!("'{}' -> {:?}", input, parse_value(input));
        }
    }
}
