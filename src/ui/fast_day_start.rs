use iced_wgpu::TextInput;
use iced_winit::widget::{text_input, Column, Row, Space, Text};

use crate::conf::{Settings, SettingsRef};
use crate::data::{Action, ActiveDay, DayStart, Location, TimedAction};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::time_limit::{check_any_limit_overlaps, InvalidTime, TimeLimit, TimeResult};
use crate::ui::top_bar::TopBar;
use crate::ui::{day_info_message, style, unbooked_time, MainView, Message, QElement, StayActive};
use crate::util::Timeline;

#[derive(Clone, Debug)]
pub enum FastDayStartMessage {
    TextChanged(String),
}

pub(super) struct FastDayStart {
    top_bar: TopBar,
    text: String,
    text_state: text_input::State,
    value: Option<DayStart>,
    limits: Vec<TimeLimit>,
    builder: DayStartBuilder,
    timeline: Timeline,
}

impl FastDayStart {
    pub fn for_work_day(settings: SettingsRef, work_day: Option<&ActiveDay>) -> Box<Self> {
        let timeline = settings.load().timeline.clone();
        let limits = unbooked_time(work_day);
        Box::new(FastDayStart {
            top_bar: TopBar {
                title: "Start day",
                help_text: "[h|o] [+|-]hours or minute",
                info: day_info_message(work_day),
                settings,
            },
            text: String::new(),
            text_state: text_input::State::focused(),
            value: Some(DayStart {
                location: Location::Office,
                ts: timeline.time_now(),
            }),
            limits,
            builder: DayStartBuilder {
                ts: TimeResult::Valid(timeline.time_now()),
                location: ParseResult::Valid(Location::Office),
            },
            timeline,
        })
    }
    fn update_text(&mut self, new_value: String) -> Option<Message> {
        self.builder
            .parse_value(&self.timeline, &self.limits, &new_value);
        self.text = new_value;
        self.value = self.builder.try_build(&self.timeline);
        None
    }
}

#[derive(Debug, Default)]
pub struct DayStartBuilder {
    location: ParseResult<Location, ()>,
    ts: TimeResult,
}

impl DayStartBuilder {
    pub fn try_build(&self, timeline: &Timeline) -> Option<DayStart> {
        let location = self.location.clone().or_default().get();
        let ts = self.ts.clone().or(timeline.time_now()).get();

        if let (Some(location), Some(ts)) = (location, ts) {
            Some(DayStart { location, ts })
        } else {
            None
        }
    }

    pub fn parse_value(&mut self, timeline: &Timeline, limits: &[TimeLimit], text: &str) {
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

        let (location, text) = parse_location(text);

        self.location = location;

        let result = crate::parsing::parse_day_end(timeline.time_now(), text);

        let result = result
            .map_invalid(|_| InvalidTime::Bad)
            .and_then(|r| check_any_limit_overlaps(r, limits));

        self.ts = result;
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

        let input_widget = TextInput::new(&mut self.text_state, "now", &self.text, move |input| {
            on_input_change(input)
        });

        let status_row = Row::with_children(vec![
            Text::new(loc_str).into(),
            Space::with_width(style::SPACE).into(),
            Text::new(time_str).into(),
        ]);

        Column::with_children(vec![
            self.top_bar.view(),
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
            Message::SubmitCurrent(stay_active) => on_submit(stay_active, self.value.as_ref()),
            Message::StoreSuccess(stay_active) => stay_active.on_main_view_store(),
            _ => None,
        }
    }
}

fn on_input_change(text: String) -> Message {
    Message::Fds(FastDayStartMessage::TextChanged(text))
}

fn on_submit(stay_active: StayActive, value: Option<&DayStart>) -> Option<Message> {
    value.map(|v| Message::StoreAction(stay_active, Action::DayStart(v.clone())))
}

#[cfg(test)]
mod test {
    use crate::conf::into_settings_ref;
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
        let settings = into_settings_ref(Settings::default().with_timeline(timeline));
        for input in i {
            let mut fds = FastDayStart::for_work_day(settings.clone(), None);
            fds.builder.parse_value(&fds.timeline, &fds.limits, *input);

            eprintln!(
                "'{}' -> {:?}",
                input,
                fds.builder.try_build(&settings.load().timeline)
            );
        }
    }
}
