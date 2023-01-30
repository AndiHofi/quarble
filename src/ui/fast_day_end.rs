use iced_native::widget::text_input::Id;
use iced_native::widget::TextInput;
use iced_winit::widget::{text_input, Column, Row, Text};

use crate::conf::SettingsRef;
use crate::data::{Action, ActiveDay, DayEnd};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::time_limit::{check_any_limit_overlaps, InvalidTime, TimeRange, TimeResult};
use crate::ui::my_text_input::MyTextInput;
use crate::ui::single_edit_ui::{FocusableUi, SingleEditUi};
use crate::ui::top_bar::TopBar;
use crate::ui::util::v_space;
use crate::ui::{day_info_message, style, unbooked_time, MainView, Message, QElement};

#[derive(Clone, Debug)]
pub enum FastDayEndMessage {
    TextChanged(String),
}

pub struct FastDayEnd {
    top_bar: TopBar,
    text: MyTextInput,
    value: Option<DayEnd>,
    limits: Vec<TimeRange>,
    builder: DayEndBuilder,
    bad_input: bool,
    original_entry: Option<DayEnd>,
    settings: SettingsRef,
}

impl FastDayEnd {
    pub fn for_work_day(settings: SettingsRef, work_day: Option<&ActiveDay>) -> Box<Self> {
        let limits = unbooked_time(work_day);
        let timeline = &settings.load().timeline;
        Box::new(Self {
            top_bar: TopBar {
                title: "Day end:",
                help_text: "[+|-]hours or minute",
                info: day_info_message(work_day),
                settings: settings.clone(),
            },
            text: MyTextInput::new(String::new(), |_| true),
            value: Some(DayEnd {
                ts: timeline.time_now(),
            }),
            limits,
            builder: DayEndBuilder {
                ts: ParseResult::Valid(timeline.time_now()),
            },
            bad_input: false,
            original_entry: None,
            settings,
        })
    }
}

impl SingleEditUi<DayEnd> for FastDayEnd {
    fn as_text(&self, orig: &DayEnd) -> String {
        orig.ts.to_string()
    }

    fn set_orig(&mut self, orig: DayEnd) {
        let txt = self.as_text(&orig);
        self.original_entry = Some(orig);
        self.update_default_input(txt);
    }

    fn try_build(&self) -> Option<DayEnd> {
        self.builder.try_build()
    }

    fn update_input(&mut self, _id: text_input::Id, input: String) -> Option<Message> {
        self.text.text = input;

        self.bad_input = false;

        let timeline = &self.settings.load().timeline;
        let (mut result, rest) = Time::parse_with_offset(timeline, &self.text.text);
        if !rest.trim_start().is_empty() {
            result = ParseResult::Invalid(());
        }

        let result = result
            .map_invalid(|_| InvalidTime::Bad)
            .and_then(|r| check_any_limit_overlaps(r, &self.limits));

        self.builder.ts = result;
        None
    }
}

impl FocusableUi for FastDayEnd {
    fn default_focus(&self) -> Id {
        self.text.id.clone()
    }
}

impl MainView for FastDayEnd {
    fn view(&self) -> QElement {
        let time_str = self
            .value
            .as_ref()
            .map(|e| e.ts.to_string())
            .unwrap_or_default();

        Column::with_children(vec![
            self.top_bar.view(),
            v_space(style::SPACE),
            self.text.show("now"),
            v_space(style::SPACE),
            Row::with_children(vec![Text::new(time_str).into()]).into(),
        ])
        .into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Input(id, value) => self.update_input(id, value),
            Message::SubmitCurrent(stay_active) => {
                Self::on_submit_message(self.try_build(), &mut self.original_entry, stay_active)
            }
            Message::StoreSuccess(stay_active) => stay_active.on_main_view_store(),
            _ => None,
        }
    }
}

fn on_input_change_message(text: String) -> Message {
    Message::Fde(FastDayEndMessage::TextChanged(text))
}

#[derive(Debug)]
struct DayEndBuilder {
    ts: TimeResult,
}

impl DayEndBuilder {
    fn try_build(&self) -> Option<DayEnd> {
        self.ts.get_ref().cloned().map(|ts| DayEnd { ts })
    }
}

#[cfg(test)]
mod test {
    use crate::data::test_support::time;
    use crate::data::{Action, ActiveDay, DayEnd, Location};

    use crate::parsing::time::Time;
    use crate::ui::fast_day_end::{FastDayEnd, FastDayEndMessage};
    use crate::ui::single_edit_ui::SingleEditUi;
    use crate::ui::stay_active::StayActive;
    use crate::ui::{MainView, Message};
    use crate::util::{StaticTimeline, TimelineProvider};
    use crate::Settings;

    #[test]
    fn test_parse_value() {
        p(&[
            ("12", Some(time("12"))),
            ("12h", None),
            ("12m", None),
            ("+1h", Some(time("13"))),
            ("-1m", Some(time("11:59"))),
            ("-15", Some(time("11:45"))),
            ("+1h15m", Some(time("13:15"))),
            ("+0m ", Some(time("12:00"))),
            ("+1", Some(time("12:01"))),
        ])
    }

    fn p(i: &[(&str, Option<Time>)]) {
        let timeline = StaticTimeline::parse("2022-01-31 12:00");
        let today = timeline.today();
        let settings = Settings {
            timeline: timeline.into(),
            ..Settings::default()
        }
        .into_settings_ref();

        let mut fde = FastDayEnd::for_work_day(
            settings,
            Some(&ActiveDay::new(today, Location::Office, None)),
        );
        for (input, expected_time) in i {
            let expected = expected_time.map(|ts| DayEnd { ts });
            let result = fde.convert_input(input);
            assert_eq!(result, expected, "'{}' -> {:?}", input, result);
        }

        for (input, expected) in i {
            fde.update(Message::Fde(FastDayEndMessage::TextChanged(
                input.to_string(),
            )));
            let result = fde.update(Message::SubmitCurrent(StayActive::Yes));
            match result {
                Some(Message::StoreAction(_, Action::DayEnd(DayEnd { ts }))) => {
                    assert_eq!(&Some(ts), expected, "For input {input}")
                }
                None => assert_eq!(&None, expected),
                other => panic!("Unexpected: {:?}", other),
            }
        }
    }
}
