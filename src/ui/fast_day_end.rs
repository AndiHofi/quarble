use std::collections::BTreeSet;

use iced_wgpu::TextInput;
use iced_winit::widget::{text_input, Column, Row, Text};

use crate::conf::{Settings, SettingsRef};
use crate::data::{Action, ActiveDay, DayEnd};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time_limit::{check_any_limit_overlaps, InvalidTime, TimeLimit, TimeResult};
use crate::ui::top_bar::TopBar;
use crate::ui::util::v_space;
use crate::ui::{
    day_info_message, min_max_booked, style, unbooked_time, MainView, Message, QElement, StayActive,
};

#[derive(Clone, Debug)]
pub enum FastDayEndMessage {
    TextChanged(String),
}

pub(super) struct FastDayEnd {
    top_bar: TopBar,
    text: String,
    text_state: text_input::State,
    value: Option<DayEnd>,
    limits: Vec<TimeLimit>,
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
            text: String::new(),
            text_state: text_input::State::focused(),
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

    pub fn entry_to_edit(&mut self, to_edit: DayEnd) -> Option<Message> {
        self.update_text(format!("{}", to_edit.ts));
        self.original_entry = Some(to_edit);
        None
    }

    fn update_text(&mut self, new_value: String) -> Option<Message> {
        self.parse_value(&new_value);
        self.text = new_value;
        self.value = self.builder.try_build();
        None
    }

    fn parse_value(&mut self, text: &str) {
        self.bad_input = false;

        let result = crate::parsing::parse_day_end(self.settings.load().timeline.time_now(), text);

        let result = result
            .map_invalid(|_| InvalidTime::Bad)
            .and_then(|r| check_any_limit_overlaps(r, &self.limits));

        self.builder.ts = result;
    }
}

fn end_day_message(actions: &BTreeSet<Action>) -> String {
    match min_max_booked(actions) {
        (None, None) => "End working day".to_string(),
        (Some(start), None) | (None, Some(start)) => format!("Last action on {}", start),
        (Some(start), Some(end)) => format!("Already booked from {} to {}", start, end),
    }
}

impl MainView for FastDayEnd {
    fn view(&mut self, _settings: &Settings) -> QElement {
        let time_str = self
            .value
            .as_ref()
            .map(|e| e.ts.to_string())
            .unwrap_or_default();

        Column::with_children(vec![
            self.top_bar.view(),
            v_space(style::SPACE),
            TextInput::new(&mut self.text_state, "now", &self.text, move |input| {
                on_input_change_message(input)
            })
            .into(),
            v_space(style::SPACE),
            Row::with_children(vec![Text::new(time_str).into()]).into(),
        ])
        .into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Fde(msg) => match msg {
                FastDayEndMessage::TextChanged(new_value) => self.update_text(new_value),
            },
            Message::SubmitCurrent(stay_active) => {
                on_submit(stay_active, &mut self.original_entry, self.value.as_ref())
            }
            Message::StoreSuccess(stay_active) => stay_active.on_main_view_store(),
            _ => None,
        }
    }
}

fn on_input_change_message(text: String) -> Message {
    Message::Fde(FastDayEndMessage::TextChanged(text))
}

fn on_submit(
    stay_active: StayActive,
    orig_value: &mut Option<DayEnd>,
    value: Option<&DayEnd>,
) -> Option<Message> {
    if let Some(v) = value {
        if let Some(orig) = std::mem::take(orig_value) {
            Some(Message::ModifyAction {
                stay_active,
                orig: Box::new(Action::DayEnd(orig)),
                update: Box::new(Action::DayEnd(v.clone())),
            })
        } else {
            Some(Message::StoreAction(stay_active, Action::DayEnd(v.clone())))
        }
    } else {
        None
    }
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
    use chrono::Timelike;

    use crate::conf::into_settings_ref;
    use crate::parsing::time::Time;
    use crate::ui::fast_day_end::FastDayEnd;
    use crate::util::{DefaultTimeline, TimelineProvider};
    use crate::Settings;

    #[test]
    fn test_parse_value() {
        let c_time = DefaultTimeline.naive_now();
        eprintln!("{:?}, {}, {}", c_time, c_time.hour(), c_time.minute());
        let time: Time = c_time.into();
        eprintln!("{}", time);
        eprintln!("{}", Time::hm(1, 9));
        p(&[
            "12", "12h", "12m", "+1h", "-1m", "-15", "-15", "+1h15m", "+0m ", "+1",
        ])
    }

    fn p(i: &[&str]) {
        let settings = into_settings_ref(Settings::default());
        let mut fde = FastDayEnd::for_work_day(settings, None);
        for input in i {
            fde.parse_value(input);
            eprintln!("'{}' -> {:?}", input, fde.builder);
        }
    }
}
