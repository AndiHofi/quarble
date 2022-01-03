use iced_wgpu::TextInput;
use iced_winit::widget::{text_input, Column, Row, Space, Text};

use crate::conf::Settings;
use crate::data::{Action, ActiveDay, DayStart, Location, TimedAction};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::time_limit::{check_limits, InvalidTime, TimeLimit, TimeResult};
use crate::ui::{input_message, style, unbooked_time_for_day, MainView, Message, QElement};
use crate::util::Timeline;

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
    builder: DayStartBuilder,
    bad_input: bool,
    timeline: Timeline,
}

impl FastDayStart {
    pub fn for_work_day(settings: &Settings, work_day: Option<&ActiveDay>) -> Box<Self> {
        let (message, limits) = if let Some(work_day) = work_day {
            let actions = work_day.actions();
            (
                input_message("Start working day", actions),
                unbooked_time_for_day(actions),
            )
        } else {
            ("Start working day".to_string(), Vec::default())
        };
        let timeline = settings.timeline.clone();
        Box::new(FastDayStart {
            text: String::new(),
            text_state: text_input::State::focused(),
            value: Some(DayStart {
                location: Location::Office,
                ts: timeline.naive_now(),
            }),
            message,
            limits,
            builder: DayStartBuilder {
                ts: TimeResult::Valid(timeline.time_now()),
                location: ParseResult::Valid(Location::Office),
            },
            bad_input: false,
            timeline,
        })
    }
    fn update_text(&mut self, new_value: String) -> Option<Message> {
        self.parse_value(&new_value);
        self.text = new_value;
        self.value = self.builder.try_build(&self.timeline);
        None
    }

    fn parse_value(&mut self, text: &str) {
        fn parse_location(text: &str) -> (ParseResult<Location, ()>, &str) {
            let text = text.trim();
            let (location, text) = if text.starts_with(&['h', 'H'][..]) {
                (ParseResult::Valid(Location::Home), (&text[1..]).trim())
            } else if text.starts_with(&['o', 'O'][..]) {
                (ParseResult::Valid(Location::Office), (&text[1..]).trim())
            } else {
                (ParseResult::None, text)
            };
            (location, text)
        }

        self.bad_input = false;
        let (location, text) = parse_location(text);

        self.builder.location = location;

        let result = crate::parsing::parse_input(self.timeline.time_now(), text);

        let result = result
            .map_invalid(|_| InvalidTime::Bad)
            .and_then(|r| check_limits(r, &self.limits));

        self.builder.ts = result;
    }
}

#[derive(Debug, Default)]
struct DayStartBuilder {
    location: ParseResult<Location, ()>,
    ts: TimeResult,
}

impl DayStartBuilder {
    fn try_build(&self, timeline: &Timeline) -> Option<DayStart> {
        let location = self.location.clone().or_default().get();
        let ts = self
            .ts
            .clone()
            .or(timeline.time_now())
            .get()
            .map(|t| t.into());

        if let (Some(location), Some(ts)) = (location, ts) {
            Some(DayStart { location, ts })
        } else {
            None
        }
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
    fn view(&mut self, _settings: &Settings) -> QElement {
        let loc_str = match self.builder.location.as_ref() {
            ParseResult::Valid(t) => t.to_string(),
            ParseResult::Invalid(_) | ParseResult::Incomplete => "Invalid location".to_string(),
            ParseResult::None => Location::Office.to_string(),
        };

        let time_str = match self.builder.ts.as_ref() {
            ParseResult::Valid(t) => t.to_string(),
            ParseResult::Invalid(e) => format!("{:?}", e),
            ParseResult::Incomplete => "invalid".to_string(),
            ParseResult::None => "now".to_string(),
        };

        let header_row = Text::new(format!(
            "Day start: [h|o] [+|-]hours or minute: {}",
            &self.message
        ));

        let input_widget = TextInput::new(&mut self.text_state, "now", &self.text, move |input| {
            on_input_change(input)
        })
        .on_submit(on_submit_message(self.value.as_ref()));

        let status_row = Row::with_children(vec![
            Text::new(loc_str).into(),
            Space::with_width(style::SPACE).into(),
            Text::new(time_str).into(),
        ]);

        Column::with_children(vec![
            header_row.into(),
            Space::with_height(style::SPACE).into(),
            input_widget.into(),
            Space::with_height(style::SPACE).into(),
            status_row.into(),
        ])
        .into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Fds(msg) => match msg {
                FastDayStartMessage::TextChanged(new_value) => self.update_text(new_value),
            },
            Message::StoreSuccess => Some(Message::Exit),
            _ => None,
        }
    }
}

fn on_input_change(text: String) -> Message {
    Message::Fds(FastDayStartMessage::TextChanged(text))
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
    use crate::ui::fast_day_start::FastDayStart;
    use crate::util::StaticTimeline;
    use crate::Settings;

    #[test]
    fn test_parse_value() {
        p(&[
            "h12", "h12h", "h12m", "h+1h", "h-1m", "h-15", "-15", "o +1h15m", "h+0m ", "h+1",
        ])
    }

    fn p(i: &[&str]) {
        let timeline = StaticTimeline::parse("2021-12-29 10:00");
        let settings = Settings::default().with_timeline(timeline);
        for input in i {
            let mut fds = FastDayStart::for_work_day(&settings, None);
            fds.parse_value(*input);

            eprintln!(
                "'{}' -> {:?}",
                input,
                fds.builder.try_build(&settings.timeline)
            );
        }
    }
}
