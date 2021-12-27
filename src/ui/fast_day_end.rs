use iced_wgpu::TextInput;
use iced_winit::widget::{text_input, Column, Row, Space, Text};

use crate::conf::Settings;
use crate::data::{Action, ActiveDay, DayEnd, TimedAction};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::time_limit::{check_limits, InvalidTime, TimeLimit, TimeResult};
use crate::ui::{style, MainView, Message, QElement};
use crate::util;

#[derive(Clone, Debug)]
pub enum FastDayEndMessage {
    TextChanged(String),
}

pub(super) struct FastDayEnd {
    text: String,
    text_state: text_input::State,
    value: Option<DayEnd>,
    message: String,
    limits: Vec<TimeLimit>,
    builder: DayEndBuilder,
    bad_input: bool,
}

impl FastDayEnd {
    pub fn for_work_day(work_day: Option<&ActiveDay>) -> Box<Self> {
        let (message, limits) = if let Some(work_day) = work_day {
            let mut actions = work_day.actions().to_vec();
            actions.sort();
            (
                end_day_message(&actions),
                valid_time_limits_for_day_end(&actions),
            )
        } else {
            ("Start working day".to_string(), Vec::default())
        };
        Box::new(Self {
            text: String::new(),
            text_state: text_input::State::focused(),
            value: Some(DayEnd {
                ts: util::time_now(),
            }),
            message,
            limits,
            builder: DayEndBuilder {
                ts: ParseResult::Valid(util::time_now().into()),
            },
            bad_input: false,
        })
    }

    fn update_text(&mut self, new_value: String) -> Option<Message> {
        self.parse_value(&new_value);
        self.text = new_value;
        self.value = self.builder.try_build();
        None
    }

    fn parse_value(&mut self, text: &str) {
        self.bad_input = false;

        let result = crate::parsing::parse_input(util::time_now().into(), text);

        let result = result
            .map_invalid(|_| InvalidTime::Bad)
            .and_then(|r| check_limits(r, &self.limits));

        self.builder.ts = result;
    }
}

fn end_day_message(actions: &[Action]) -> String {
    match min_max_booked(actions) {
        (None, None) => "End working day".to_string(),
        (Some(start), None) | (None, Some(start)) => format!("Last action on {}", start),
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

fn valid_time_limits_for_day_end(actions: &[Action]) -> Vec<TimeLimit> {
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

impl MainView for FastDayEnd {
    fn new() -> Box<Self> {
        Box::new(FastDayEnd {
            text: String::new(),
            text_state: text_input::State::focused(),
            value: Some(DayEnd {
                ts: util::time_now(),
            }),
            message: "Start working day".to_string(),
            limits: Vec::default(),
            builder: DayEndBuilder {
                ts: ParseResult::Valid(util::time_now().into()),
            },
            bad_input: false,
        })
    }

    fn view(&mut self, _settings: &Settings) -> QElement {
        let time_str = self
            .value
            .as_ref()
            .map(|e| e.ts.to_string())
            .unwrap_or_default();

        Column::with_children(vec![
            Text::new(format!("Day end: [+|-]hours or minute: {}", &self.message)).into(),
            Space::with_width(style::SPACE).into(),
            TextInput::new(&mut self.text_state, "now", &self.text, move |input| {
                on_input_change(input)
            })
            .on_submit(on_submit_message(self.value.as_ref()))
            .into(),
            Space::with_width(style::SPACE).into(),
            Row::with_children(vec![Text::new(time_str).into()]).into(),
        ])
        .into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Fde(msg) => match msg {
                FastDayEndMessage::TextChanged(new_value) => self.update_text(new_value),
            },
            Message::StoreSuccess => Some(Message::Exit),
            _ => None,
        }
    }
}

fn on_input_change(text: String) -> Message {
    Message::Fde(FastDayEndMessage::TextChanged(text))
}

fn on_submit_message(value: Option<&DayEnd>) -> Message {
    if let Some(v) = value {
        Message::StoreAction(Action::DayEnd(v.clone()))
    } else {
        Message::Update
    }
}

#[derive(Debug)]
struct DayEndBuilder {
    ts: TimeResult,
}

impl DayEndBuilder {
    fn try_build(&self) -> Option<DayEnd> {
        self.ts.get_ref().map(|ts| DayEnd { ts: (*ts).into() })
    }
}

#[cfg(test)]
mod test {
    use crate::parsing::time::Time;
    use crate::ui::fast_day_end::FastDayEnd;
    use crate::ui::MainView;
    use chrono::Timelike;

    use crate::util;

    #[test]
    fn test_parse_value() {
        let c_time = util::time_now();
        eprintln!("{:?}, {}, {}", c_time, c_time.hour(), c_time.minute());
        let time: Time = c_time.into();
        eprintln!("{}", time);
        eprintln!("{}", Time::hm(1, 9));
        p(&[
            "12", "12h", "12m", "+1h", "-1m", "-15", "-15", "+1h15m", "+0m ", "+1",
        ])
    }

    fn p(i: &[&str]) {
        let mut fde = FastDayEnd::new();
        for input in i {
            fde.parse_value(input);
            eprintln!("'{}' -> {:?}", input, fde.builder);
        }
    }
}
