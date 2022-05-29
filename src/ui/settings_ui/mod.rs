use std::collections::BTreeMap;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

use iced_core::Length;
use iced_native::widget::text_input::State;
use iced_native::widget::{button, Button, Container, scrollable, Scrollable, text_input};
use iced_native::widget::{Column, Row};
use regex::Regex;

use crate::ui::my_text_input::MyTextInput;
use shortcut_ui::ShortCutUi;

use crate::conf::{BreaksConfig, SettingsRef};
use crate::data::JiraIssue;
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::time_relative::TimeRelative;
use crate::parsing::JiraIssueParser;
use crate::ui::focus_handler::FocusHandler;
use crate::ui::util::{h_space, v_space};
use crate::ui::{MainView, Message, QElement, style, text};
use crate::{Settings, SettingsSer};

mod shortcut_ui;

#[derive(Clone, Debug)]
pub enum SettingsUIMessage {
    AddShortcut,
    ResetSettings,
    SubmitSettings,
}

pub struct SettingsUI {
    settings: SettingsRef,
    original: SettingsSer,
    db_dir: MyTextInput,
    resolution: MyTextInput,
    min_breaks: MyTextInput,
    min_work: MyTextInput,
    default_break_start: MyTextInput,
    default_break_end: MyTextInput,
    max_recent_issues: MyTextInput,
    shortcuts: Vec<ShortCutUi>,
    shortcuts_scroll: scrollable::State,
    add_shortcut_button: button::State,
    submit_button: button::State,
    reset_button: button::State,
    settings_changed: bool,
}

impl SettingsUI {
    pub fn new(settings: SettingsRef) -> Box<Self> {
        let settings_v: &Settings = &**settings.load();
        let original = SettingsSer::from_settings(settings_v);
        let o = SettingsSer::from_settings(settings_v);
        let shortcuts = Vec::from_iter(o.issue_shortcuts.iter().map(|(sc, i)| ShortCutUi {
            shortcut: MyTextInput::new(*sc, accept_shortcut),
            id: MyTextInput::new(i.ident.as_str(), accept_issue_id),
            description: MyTextInput::new(i.description.as_deref().unwrap_or_default(), no_check),
            default_action: MyTextInput::new(
                i.default_action.as_deref().unwrap_or_default(),
                no_check,
            ),
        }));
        let mut max_recent_issues = MyTextInput::new(o.max_recent_issues, accept_number);
        max_recent_issues.input.focus();
        Box::new(Self {
            settings,
            original,
            db_dir: MyTextInput::new(o.db_dir.to_string_lossy(), no_check),
            resolution: MyTextInput::new(o.resolution_minutes, accept_number),
            min_breaks: MyTextInput::new(o.breaks.min_breaks_minutes, accept_number),
            min_work: MyTextInput::new(o.breaks.min_work_time_minutes, accept_number),
            default_break_start: MyTextInput::new(o.breaks.default_break.0, accept_time),
            default_break_end: MyTextInput::new(o.breaks.default_break.1, accept_time),
            max_recent_issues,
            shortcuts,
            shortcuts_scroll: scrollable::State::new(),
            add_shortcut_button: button::State::new(),
            submit_button: button::State::new(),
            reset_button: button::State::new(),
            settings_changed: false,
        })
    }

    fn update_text(&mut self, text: String) -> Option<Message> {
        if self.db_dir.is_focused() {
            self.db_dir.text = text;
        } else if self.resolution.is_focused() {
            self.resolution.accept_input(text);
        } else if self.max_recent_issues.is_focused() {
            self.max_recent_issues.accept_input(text);
        } else if self.default_break_start.is_focused() {
            self.default_break_start.accept_input(text);
        } else if self.default_break_end.is_focused() {
            self.default_break_end.accept_input(text);
        } else if self.min_breaks.is_focused() {
            self.min_breaks.accept_input(text);
        } else if self.min_work.is_focused() {
            self.min_work.accept_input(text);
        } else {
            for sc in self.shortcuts.iter_mut() {
                if sc.shortcut.is_focused() {
                    sc.shortcut.accept_input(text);
                    break;
                } else if sc.id.is_focused() {
                    sc.id.accept_input(text);
                    break;
                } else if sc.description.is_focused() {
                    sc.description.accept_input(text);
                    break;
                } else if sc.default_action.is_focused() {
                    sc.default_action.accept_input(text);
                    break;
                }
            }
        }

        None
    }

    fn validate(&mut self) -> Option<SettingsSer> {
        fn validate_db_dir(input: &MyTextInput, orig: &SettingsSer) -> VResult<PathBuf> {
            let db_dir = PathBuf::from(&input.text);
            if db_dir != orig.db_dir {
                if !db_dir.is_dir() {
                    Err("Directory does not exist".to_string())
                } else {
                    Ok(db_dir)
                }
            } else {
                Ok(orig.db_dir.clone())
            }
        }

        fn validate_max_recent(input: &MyTextInput) -> VResult<u32> {
            match u32::from_str(&input.text) {
                Ok(max_recent) => {
                    if max_recent == 0 {
                        Err("Must be >= 1".to_string())
                    } else if max_recent > 100 {
                        Err("For performance reasons must be <= 100".to_string())
                    } else {
                        Ok(max_recent)
                    }
                }
                Err(_) => Err("Invalid".to_string()),
            }
        }

        fn validate_num(input: &MyTextInput, max: u32) -> VResult<u32> {
            match u32::from_str(&input.text) {
                Ok(v) if v <= max => Ok(v),
                Ok(_) => Err(format!("Value must be <= {max}")),
                Err(_) => Err("invalid".to_string()),
            }
        }

        fn validate_default_break_start(
            input: &MyTextInput,
            breaks_duration: &VResult<u32>,
        ) -> VResult<Time> {
            let r = Time::parse_prefix(&input.text);
            match r {
                (_, rest) if !rest.is_empty() => Err("Bad input".to_string()),
                (ParseResult::Invalid(_) | ParseResult::Incomplete, _) => {
                    Err("Bad input".to_string())
                }
                (ParseResult::None, _) if matches!(breaks_duration, Ok(0)) => Ok(Time::ZERO),
                (ParseResult::None, _) => Err("Missing break start".to_string()),
                (ParseResult::Valid(t), _) => Ok(t),
            }
        }

        fn validate_default_break_end(
            input: &MyTextInput,
            start: &VResult<Time>,
            duration: &VResult<u32>,
        ) -> VResult<Time> {
            match Time::parse_prefix(&input.text) {
                _ if start.is_err() => Ok(Time::ZERO),
                _ if matches!(duration, Ok(0)) => Ok(Time::ZERO),
                (_, rest) if !rest.is_empty() => Err("Bad input".to_string()),
                (ParseResult::Invalid(_) | ParseResult::Incomplete, _) => {
                    Err("Bad input".to_string())
                }
                (ParseResult::Valid(end), _) => match (start, duration) {
                    (&Ok(start), &Ok(duration))
                        if start + TimeRelative::from_minutes_sat(duration as i32) != end =>
                    {
                        Err(format!(
                            "Start {start} and duration {duration} do not match to this"
                        ))
                    }
                    _ => Ok(end),
                },
                (ParseResult::None, _) => match (start, duration) {
                    (&Ok(start), &Ok(duration)) => {
                        Ok(start + TimeRelative::from_minutes_sat(duration as i32))
                    }
                    _ => Err("Missing input".to_string()),
                },
            }
        }

        let db_dir = validate_db_dir(&self.db_dir, &self.original);
        let max_recent = validate_max_recent(&self.max_recent_issues);
        let breaks_dur = validate_num(&self.min_breaks, 6 * 60);
        let min_work = validate_num(&self.min_work, 12 * 60);
        let break_start = validate_default_break_start(&self.default_break_start, &breaks_dur);
        let break_end =
            validate_default_break_end(&self.default_break_end, &break_start, &breaks_dur);
        let resolution = validate_num(&self.resolution, 60);
        let shortcuts = self.validate_shortcuts();

        let db_dir = self.db_dir.consume_err(db_dir);
        let max_recent = self.max_recent_issues.consume_err(max_recent);
        let breaks_dur = self.min_breaks.consume_err(breaks_dur);
        let min_work = self.min_work.consume_err(min_work);
        let break_start = self.default_break_start.consume_err(break_start);
        let break_end = self.default_break_end.consume_err(break_end);
        let resolution = self.resolution.consume_err(resolution);

        let breaks = match (breaks_dur, min_work, break_start, break_end) {
            (Ok(dur), Ok(mw), Ok(s), Ok(e)) => Some(BreaksConfig {
                min_breaks_minutes: dur,
                min_work_time_minutes: mw,
                default_break: (s, e),
            }),
            _ => None,
        };

        match (db_dir, resolution, max_recent, breaks, shortcuts) {
            (
                Ok(db_dir),
                Ok(resolution_minutes),
                Ok(max_recent_issues),
                Some(breaks),
                Some(issue_shortcuts),
            ) => Some(SettingsSer {
                db_dir,
                resolution_minutes,
                issue_shortcuts,
                breaks,
                max_recent_issues,
            }),
            _ => None,
        }
    }

    fn validate_shortcuts(&mut self) -> Option<BTreeMap<char, JiraIssue>> {
        fn validate_issue_id(input: &MyTextInput) -> VResult<String> {
            if JiraIssueParser::valid_id(&input.text) {
                Ok(input.text.clone())
            } else {
                Err("Invalid id".to_string())
            }
        }

        fn empty_to_none(s: &str) -> Option<String> {
            let trim = s.trim();
            if trim.is_empty() {
                None
            } else {
                Some(trim.to_string())
            }
        }

        let mut result = BTreeMap::new();

        for ShortCutUi {
            shortcut,
            id,
            description,
            default_action,
        } in &mut self.shortcuts
        {
            if shortcut.text.is_empty() {
                continue;
            }

            let sc = shortcut.text.chars().next().unwrap();
            let issue_id = validate_issue_id(id);
            let issue_id = id.consume_err(issue_id);

            let sc = if result.contains_key(&sc) {
                Err(format!("Duplicate id {sc}"))
            } else {
                Ok(sc)
            };
            let sc = shortcut.consume_err(sc);

            if let (Ok(sc), Ok(issue_id)) = (sc, issue_id) {
                let issue = JiraIssue {
                    ident: issue_id,
                    description: empty_to_none(&description.text),
                    default_action: empty_to_none(&default_action.text),
                };
                result.insert(sc, issue);
            }
        }

        Some(result)
    }
}

type VResult<T> = Result<T, String>;

impl<'a> FocusHandler<'a, Vec<&'a mut text_input::State>> for SettingsUI {
    fn focus_order(&'a mut self) -> Vec<&'a mut State> {
        let mut result = vec![
            &mut self.db_dir.input,
            &mut self.resolution.input,
            &mut self.max_recent_issues.input,
            &mut self.min_breaks.input,
            &mut self.min_work.input,
            &mut self.default_break_start.input,
            &mut self.default_break_end.input,
        ];
        for e in &mut self.shortcuts {
            result.push(&mut e.shortcut.input);
            result.push(&mut e.id.input);
            result.push(&mut e.description.input);
            result.push(&mut e.default_action.input);
        }
        result
    }
}

impl MainView for SettingsUI {
    fn view(&mut self) -> QElement {
        let breaks_dur = Row::with_children(vec![
            self.min_breaks
                .show_with_input_width("Required break (Minutes):", Length::Units(60)),
            h_space(style::DSPACE),
            self.min_work
                .show_with_input_width("Work time requiring break (Minutes):", Length::Units(60)),
        ]);

        let breaks_time = Row::with_children(vec![
            self.default_break_start
                .show_with_input_width("Default break start (hh:mm):", Length::Units(60)),
            h_space(style::DSPACE),
            self.default_break_end
                .show_with_input_width("Default break end (hh:mm):", Length::Units(60)),
        ]);

        let mut shortcuts = Scrollable::new(&mut self.shortcuts_scroll)
            .width(Length::Fill)
            .padding(style::WINDOW_PADDING)
            .spacing(4);
        for sc in self.shortcuts.iter_mut() {
            shortcuts = shortcuts.push(sc.show());
        }
        let shortcuts = Container::new(shortcuts)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::ContentStyle);

        let mut reset_button = Button::new(&mut self.reset_button, text("Reset")).style(style::Tab);
        if self.settings_changed {
            reset_button =
                reset_button.on_press(Message::SettingsUi(SettingsUIMessage::ResetSettings))
        }

        let submit_button = Button::new(&mut self.submit_button, text("Submit"))
            .style(style::Tab)
            .on_press(Message::SettingsUi(SettingsUIMessage::SubmitSettings));

        let content = Column::with_children(vec![
            v_space(style::SPACE),
            Row::with_children(vec![
                self.db_dir
                    .show_with_input_width("Storage directory:", Length::Units(400)),
                h_space(Length::Fill),
                submit_button.into(),
                h_space(style::SPACE),
                reset_button.into(),
            ])
            .into(),
            v_space(style::SPACE),
            self.resolution
                .show_with_input_width("Booking resolution (Minutes):", Length::Units(60)),
            v_space(style::SPACE),
            self.max_recent_issues
                .show("Maximum number of recent issues:"),
            v_space(style::DSPACE),
            breaks_dur.into(),
            v_space(style::SPACE),
            breaks_time.into(),
            v_space(style::DSPACE),
            Row::with_children(vec![
                text("Configured shortcuts:"),
                h_space(Length::Fill),
                style::inline_button(&mut self.add_shortcut_button, "+")
                    .on_press(Message::SettingsUi(SettingsUIMessage::AddShortcut))
                    .into(),
            ])
            .into(),
            shortcuts.into(),
        ]);

        content.into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::TextChanged(value) => self.update_text(value),
            Message::SettingsUi(SettingsUIMessage::AddShortcut) => {
                self.shortcuts.push(ShortCutUi::empty());
                self.shortcuts.last_mut().unwrap().shortcut.input.focus();
                self.shortcuts_scroll.snap_to(1.0);
                None
            }
            Message::SettingsUi(SettingsUIMessage::ResetSettings) => {
                let settings = self.settings.clone();
                let guard = settings.load_full();
                settings.store(Arc::new(guard.apply_ser(self.original.clone())));
                *self = *SettingsUI::new(settings);
                None
            }
            Message::Next => {
                let _ = self.validate();
                self.focus_next()
            }
            Message::Previous => {
                let _ = self.validate();
                self.focus_previous()
            }
            Message::SubmitCurrent(_) | Message::SettingsUi(SettingsUIMessage::SubmitSettings) => {
                if let Some(x) = self.validate() {
                    self.settings_changed = true;
                    let updated = self.settings.load_full().apply_ser(x);
                    self.settings.store(Arc::new(updated));
                }
                None
            }
            _ => None,
        }
    }
}

fn no_check(_: &str) -> bool {
    true
}

fn valid_minutes(input: &str) -> bool {
    u32::from_str(input)
        .map(|v| v < 12 * 60)
        .unwrap_or_default()
}

fn valid_resolution(input: &str) -> bool {
    u32::from_str(input)
        .map(|v| v > 0 && v < 60)
        .unwrap_or_default()
}

fn valid_time(input: &str) -> bool {
    let (r, rest) = Time::parse_prefix(input);
    matches!(r, ParseResult::Valid(_)) && rest.is_empty()
}

fn accept_time(input: &str) -> bool {
    VALID_TIME.is_match(input)
}

fn accept_shortcut(input: &str) -> bool {
    VALID_SHORTCUT.is_match(input)
}

fn accept_issue_id(input: &str) -> bool {
    VALID_ISSUE.is_match(input)
}

fn accept_number(input: &str) -> bool {
    VALID_NUMBER.is_match(input)
}

lazy_static::lazy_static! {
    static ref VALID_NUMBER: Regex = Regex::new("^[0-9]{0,4}$").unwrap();
    static ref VALID_ISSUE: Regex = Regex::new("(^$)|(^[a-zA-Z]+(-[0-9]*)?$)").unwrap();
    static ref VALID_TIME: Regex = Regex::new("^([0-9]{1,2}:?([0-9]{0,2}))?$").unwrap();
    static ref VALID_SHORTCUT: Regex = Regex::new("^[a-zA-Z]?$").unwrap();
}
