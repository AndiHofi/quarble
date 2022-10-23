use crate::conf::{SettingsRef};
use crate::data::{Action, ActiveDay, DayStart, Location};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::time_limit::{check_any_limit_overlaps, InvalidTime, TimeRange, TimeResult};
use crate::ui::single_edit_ui::SingleEditUi;
use crate::ui::top_bar::TopBar;
use crate::ui::{day_info_message, style, unbooked_time, MainView, Message, QElement};
use crate::util::Timeline;
use iced_wgpu::TextInput;
use iced_winit::widget::{text_input, Column, Row, Space, Text};
use crate::ui::stay_active::StayActive;

#[derive(Clone, Debug)]
pub enum FastDayStartMessage {
    TextChanged(String),
}

pub struct FastDayStart {
    top_bar: TopBar,
    text: String,
    text_state: text_input::State,
    value: Option<DayStart>,
    limits: Vec<TimeRange>,
    builder: DayStartBuilder,
    timeline: Timeline,
    orig: Option<DayStart>,
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
            orig: None,
        })
    }

}

impl SingleEditUi<DayStart> for FastDayStart {
    fn as_text(&self, e: &DayStart) -> String {
        let loc = match e.location {
            Location::Office => "o",
            Location::Home => "h",
            Location::Other(ref l) => l.0.as_str(),
        };
        format!("{} {}", loc, e.ts)
    }

    fn set_orig(&mut self, orig: DayStart) {
        let input = self.as_text(&orig);
        self.orig = Some(orig);
        self.update_input(input);
    }

    fn try_build(&self) -> Option<DayStart> {
        self.builder.try_build(&self.timeline)
    }

    fn update_input(&mut self, input: String) -> Option<Message> {
        self.text = input;
        self.builder
            .parse_value(&self.timeline, &self.limits, &self.text);
        None
    }
}

impl MainView for FastDayStart {
    fn view(&mut self) -> QElement {
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
            Message::Fds(FastDayStartMessage::TextChanged(new_value)) => {
                self.update_input(new_value)
            }
            Message::SubmitCurrent(stay_active) => {
                Self::on_submit_message(self.try_build(), &mut self.orig, stay_active)
            }
            Message::StoreSuccess(stay_active) => stay_active.on_main_view_store(),
            _ => None,
        }
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

    pub fn parse_value(&mut self, timeline: &Timeline, limits: &[TimeRange], text: &str) {
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

        let (mut result, rest) = Time::parse_with_offset(timeline, text);
        if !rest.trim_start().is_empty() {
            result = ParseResult::Invalid(())
        };

        let result = result
            .map_invalid(|_| InvalidTime::Bad)
            .and_then(|r| check_any_limit_overlaps(r, limits));

        self.ts = result;
    }
}

fn on_input_change(text: String) -> Message {
    Message::Fds(FastDayStartMessage::TextChanged(text))
}

#[cfg(test)]
mod test {
    use crate::conf::into_settings_ref;
    use crate::data::test_support::time;
    use crate::data::Location::*;
    use crate::data::{Action, ActiveDay, DayStart, Location};
    use crate::ui::fast_day_start::{FastDayStart, FastDayStartMessage};
    use crate::ui::single_edit_ui::SingleEditUi;
    use crate::ui::stay_active::StayActive;
    use crate::ui::{MainView, Message};
    use crate::util::{StaticTimeline, TimelineProvider};
    use crate::Settings;
    use std::sync::Arc;

    #[test]
    fn test_parse_value() {
        let r = |l, t| {
            Some(DayStart {
                location: l,
                ts: time(t),
            })
        };

        p(&[
            ("h12", r(Home, "12")),
            ("h12h", None),
            ("o12m", None),
            ("h+12m", r(Home, "12:12")),
            ("o+12m", r(Office, "12:12")),
            ("h+1h", r(Home, "13")),
            ("h-1m", r(Home, "11:59")),
            ("h-15", r(Home, "11:45")),
            ("-15", r(Office, "11:45")),
            ("o +1h15m", r(Office, "13:15")),
            ("h+0m ", r(Home, "12")),
            ("h+1", r(Home, "12:01")),
        ])
    }

    fn p(i: &[(&str, Option<DayStart>)]) {
        let timeline = StaticTimeline::parse("2021-12-29 12:00");
        let today = timeline.today();
        let settings = into_settings_ref(Settings {
            timeline: Arc::new(timeline),
            ..Settings::default()
        });
        let mut fds = FastDayStart::for_work_day(
            settings,
            Some(&ActiveDay::new(today, Location::Office, None)),
        );
        for (input, expected) in i {
            let result = fds.convert_input(*input);

            assert_eq!(&result, expected, "For input '{input}'");
        }

        for (input, expected) in i {
            fds.update(Message::Fds(FastDayStartMessage::TextChanged(
                input.to_string(),
            )));
            let result = fds.update(Message::SubmitCurrent(StayActive::Yes));
            match result {
                Some(Message::StoreAction(_, Action::DayStart(result))) => {
                    assert_eq!(&Some(result), expected, "For input {input}")
                }
                None => assert_eq!(&None, expected, "For input {input}"),
                r => panic!("Unexpected for input {input}: {r:?}"),
            }
        }
    }
}
