use iced_core::Length;
use iced_native::widget::text_input::{Action, Id};
use iced_native::widget::{Row, Text};
use iced_winit::widget::{text_input, Column};

use crate::conf::SettingsRef;
use crate::data::{ActiveDay, CurrentWork, JiraIssue, RecentIssuesRef, Work, WorkEntry};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::IssueParserWithRecent;
use crate::ui::book_single::nparsing::{IssueInput, ValidWorkData, WTime, WorkData};
use crate::ui::clip_read::ClipRead;
use crate::ui::focus_handler::FocusHandler;
use crate::ui::my_text_input::MyTextInput;
use crate::ui::single_edit_ui::{FocusableUi, SingleEditUi};
use crate::ui::top_bar::TopBar;
use crate::ui::util::{h_space, v_space};
use crate::ui::{day_info_message, style, text, MainView, Message, QElement};

mod nparsing;
mod parsing;

#[derive(Clone, Debug)]
pub enum BookSingleMessage {
    TextChanged(String),
}

pub struct BookSingleUI {
    top_bar: TopBar,
    builder: WorkData,
    settings: SettingsRef,
    orig: Option<WorkEntry>,
    recent_issues: RecentIssuesRef,
    last_end: Option<Time>,

    start: MyTextInput,
    end: MyTextInput,
    id: MyTextInput,
    comment: MyTextInput,
    description: MyTextInput,

    has_focus: Option<text_input::Id>,
}

impl SingleEditUi<WorkEntry> for BookSingleUI {
    fn as_text(&self, e: &WorkEntry) -> String {
        match e {
            WorkEntry::Work(e) => {
                format!("{} {} {} {}", e.start, e.end, e.task.ident, e.description)
            }
            WorkEntry::Current(e) => format!("{} - {} {}", e.start, e.task.ident, e.description),
        }
    }

    fn set_orig(&mut self, orig: WorkEntry) {
        match &orig {
            WorkEntry::Work(Work {
                start,
                end,
                task: JiraIssue { ident, .. },
                description,
            }) => {
                self.start.text = start.to_string();
                self.end.text = end.to_string();
                self.id.text = ident.to_string();
                self.comment.text = description.to_string();
            }
            WorkEntry::Current(CurrentWork {
                start,
                task: JiraIssue { ident, .. },
                description,
            }) => {
                self.start.text = start.to_string();
                self.end.text = "-".to_string();
                self.id.text = ident.to_string();
                self.comment.text = description.to_string();
            }
        }
        self.orig = Some(orig);
    }

    fn try_build(&self) -> Option<WorkEntry> {
        let now = self.settings.load().timeline.time_now();
        self.builder
            .try_as_work_data(self.last_end, now)
            .map(|v| match v {
                ValidWorkData {
                    start,
                    end: None,
                    task,
                    msg,
                    description,
                } => WorkEntry::Current(CurrentWork {
                    start,
                    task: JiraIssue {
                        ident: task.to_string(),
                        description: description.map(|s| s.to_string()),
                        default_action: None,
                    },
                    description: msg.to_string(),
                }),
                ValidWorkData {
                    start,
                    end: Some(end),
                    task,
                    msg,
                    description,
                } => WorkEntry::Work(Work {
                    start,
                    end,
                    task: JiraIssue {
                        ident: task.to_string(),
                        description: description.map(|s| s.to_string()),
                        default_action: None,
                    },
                    description: msg.to_string(),
                }),
            })
    }

    fn entry_to_edit(&mut self, orig: WorkEntry) {
        match &orig {
            WorkEntry::Current(orig) => {
                self.start.text = orig.start.to_string();
                self.id.text = orig.task.ident.to_string();
                self.comment.text = orig.description.to_string();
            }
            WorkEntry::Work(orig) => {
                self.start.text = orig.start.to_string();
                self.end.text = orig.end.to_string();
                self.id.text = orig.task.ident.to_string();
                self.comment.text = orig.description.to_string();
            }
        }
        self.set_orig(orig);
    }

    fn update_input(&mut self, id: text_input::Id, input: String) -> Option<Message> {
        let f = &self.has_focus;
        let text_follow_up = if self.start.id == id {
            self.start.accept_input(input)
        } else if self.end.id == id {
            self.end.accept_input(input)
        } else if self.id.id == id {
            self.id.accept_input(input)
        } else if self.comment.id == id {
            self.comment.accept_input(input)
        } else if self.description.id == id {
            self.description.accept_input(input)
        } else {
            None
        };

        if text_follow_up.is_some() {
            return text_follow_up;
        }

        if self.id.is_focused(f) || self.comment.id == id {
            return Some(Message::FilterRecent(
                self.id.text.as_str().into(),
                self.comment.text.as_str().into(),
            ));
        }

        self.follow_up_msg()
    }
}

impl FocusableUi for BookSingleUI {
    fn default_focus(&self) -> Id {
        self.start.id.clone()
    }
}

impl BookSingleUI {
    pub fn for_active_day(
        settings: SettingsRef,
        recent_issues: RecentIssuesRef,
        active_day: Option<&ActiveDay>,
    ) -> Box<Self> {
        let now = settings.load().timeline.time_now();
        let last_end = active_day.and_then(|d| d.last_action_end(now));

        let mut result = Box::new(Self {
            top_bar: TopBar {
                title: "Book issue:",
                help_text: "(start [end])|duration <issue id> <message>",
                info: day_info_message(active_day),
                settings: settings.clone(),
            },
            builder: Default::default(),
            settings,
            orig: None,
            recent_issues,
            last_end,
            start: MyTextInput::msg_aware("", nparsing::time_input).with_placeholder("start"),
            end: MyTextInput::msg_aware("", nparsing::time_input).with_placeholder("end"),
            id: MyTextInput::msg_aware("", nparsing::issue_input).with_placeholder("Issue"),
            comment: MyTextInput::msg_aware("", nparsing::comment_input)
                .with_placeholder("Comment"),
            description: MyTextInput::new("", |_| true).with_placeholder("Description"),
            has_focus: None,
        });

        result
    }

    fn follow_up_msg(&mut self) -> Option<Message> {
        if self.builder.needs_clipboard() {
            self.builder.clipboard_reading = ClipRead::Reading;
            Some(Message::ReadClipboard)
        } else {
            None
        }
    }

    fn validate(&mut self) {
        let settings = self.settings.load();
        let timeline = &settings.timeline;
        let recent_issues = self.recent_issues.borrow();
        let issue_parser = IssueParserWithRecent::new(&settings.issue_parser, &recent_issues);
        self.builder.start = nparsing::parse_start(&self.start.text, timeline);
        self.builder.end = nparsing::parse_end(&self.end.text, timeline);
        self.builder.task = nparsing::parse_issue(&self.id.text, &issue_parser);
        let comment = self.comment.text.as_str().trim();
        self.builder.msg = if comment.is_empty() {
            None
        } else {
            Some(comment.to_string())
        };
        let description = self.description.text.trim();
        self.builder.description = if description.is_empty() {
            None
        } else {
            Some(description.to_string())
        };
    }
}


impl MainView for BookSingleUI {
    fn view(&self) -> QElement {
        let input_line = Row::with_children(vec![
            self.start
                .show_text_input(Length::Units(60))
                .into(),
            h_space(style::SPACE),
            self.end.show_text_input(Length::Units(60)).into(),
            h_space(style::SPACE),
            self.id.show_text_input(Length::Units(100)).into(),
            h_space(style::SPACE),
            self.comment.show_text_input(Length::Units(350)).into(),
            h_space(style::SPACE),
            self.description.show_text_input(Length::Fill).into(),
        ]);

        let now = self.settings.load().timeline.time_now();

        let status = Row::with_children(vec![
            text("Start:"),
            h_space(style::SPACE),
            tord_info(now, self.last_end, self.builder.start.clone()),
            h_space(style::DSPACE),
            text("End:"),
            h_space(style::SPACE),
            tord_info(now, self.last_end, self.builder.end.clone()),
            h_space(style::DSPACE),
            text("Task:"),
            h_space(style::SPACE),
            task_info(self.builder.task.as_ref(), &self.builder.clipboard_reading),
            h_space(style::DSPACE),
            text("Message:"),
            h_space(style::SPACE),
            text(self.builder.msg.as_deref().unwrap_or("<no message>")),
            h_space(style::DSPACE),
            text("Description:"),
            h_space(style::SPACE),
            text(self.builder.msg.as_deref().unwrap_or("<no description>")),
        ]);

        Column::with_children(vec![
            self.top_bar.view(),
            v_space(style::SPACE),
            input_line.into(),
            v_space(style::SPACE),
            status.into(),
        ])
        .into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Input(id, input) => self.update_input(id, input),
            Message::ClipboardValue(v) => {
                dbg!("Clipboard value", &v);
                self.builder.apply_clipboard(v);
                None
            }
            Message::SubmitCurrent(stay_active) => {
                self.validate();
                dbg!(&self.builder);
                Self::on_submit_message(self.try_build(), &mut self.orig, stay_active)
            }
            Message::StoreSuccess(stay_active) => stay_active.on_main_view_store(),
            Message::Focus(id) => {
                self.has_focus = Some(id);
                None
            }
            _ => self.follow_up_msg(),
        }
    }
}

fn task_info<'a>(v: ParseResult<&'a IssueInput, &'a ()>, clipboard: &'a ClipRead) -> QElement<'a> {
    match v {
        ParseResult::Valid(t) => task_text(t),
        ParseResult::Invalid(_) => text("invalid"),
        ParseResult::Incomplete => text("incomplete"),
        ParseResult::None => text(clipboard.as_str()),
    }
}

fn task_text(t: &IssueInput) -> QElement {
    let t = match t {
        IssueInput::Match(s) => s.as_str(),
        IssueInput::Recent(j) => j.ident.as_str(),
        IssueInput::Clipboard => "<clip>",
    };
    text(t)
}

fn tord_info<'a>(now: Time, last_end: Option<Time>, v: ParseResult<WTime, ()>) -> QElement<'a> {
    Text::new(
        v.get()
            .map(|e| match e {
                WTime::Time(t) => t.to_string(),
                WTime::Relative(r) => r.to_string(),
                WTime::Last => last_end
                    .map(|t| t.to_string())
                    .unwrap_or_else(|| "no pred".to_string()),
                WTime::Now => now.to_string(),
                WTime::Empty => "-".to_string(),
            })
            .unwrap_or_else(|| "invalid".to_string()),
    )
    .into()
}

#[cfg(test)]
mod test;
