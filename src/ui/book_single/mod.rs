use iced_native::widget::Row;
use iced_wgpu::TextInput;
use iced_winit::widget::{text_input, Column};

use parsing::WorkBuilder;

use crate::conf::SettingsRef;
use crate::data::{ActiveDay, JiraIssue, RecentIssuesRef, Work};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::ui::clip_read::ClipRead;
use crate::ui::single_edit_ui::SingleEditUi;
use crate::ui::top_bar::TopBar;
use crate::ui::util::{h_space, v_space};
use crate::ui::{day_info_message, style, text, time_info, MainView, Message, QElement};

mod parsing;

#[derive(Clone, Debug)]
pub enum BookSingleMessage {
    TextChanged(String),
}

pub struct BookSingleUI {
    top_bar: TopBar,
    input_state: text_input::State,
    input: String,
    builder: WorkBuilder,
    settings: SettingsRef,
    orig: Option<Work>,
    recent_issues: RecentIssuesRef,
    last_end: Option<Time>,
}

impl SingleEditUi<Work> for BookSingleUI {
    fn update_input(&mut self, input: String) {
        self.input = input;
        let recent = self.recent_issues.borrow();

        self.builder
            .parse_input(&self.settings.load(), &recent, self.last_end, &self.input)
    }

    fn as_text(&self, e: &Work) -> String {
        format!("{} {} {} {}", e.start, e.end, e.task.ident, e.description)
    }

    fn set_orig(&mut self, orig: Work) {
        self.orig = Some(orig);
    }

    fn try_build(&self) -> Option<Work> {
        let now = self.settings.load().timeline.time_now();
        self.builder.try_build(now)
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

        Box::new(Self {
            top_bar: TopBar {
                title: "Book issue:",
                help_text: "(start [end])|duration <issue id> <message>",
                info: day_info_message(active_day),
                settings: settings.clone(),
            },
            input_state: text_input::State::focused(),
            input: "".to_string(),
            builder: Default::default(),
            settings,
            orig: None,
            recent_issues,
            last_end,
        })
    }

    fn follow_up_msg(&mut self) -> Option<Message> {
        if self.builder.needs_clipboard() {
            self.builder.clipboard_reading = ClipRead::Reading;
            Some(Message::ReadClipboard)
        } else {
            None
        }
    }
}

impl MainView for BookSingleUI {
    fn view(&mut self) -> QElement {
        let input = TextInput::new(&mut self.input_state, "", &self.input, |s| {
            Message::Bs(BookSingleMessage::TextChanged(s))
        });

        let now = self.settings.load().timeline.time_now();

        let status = Row::with_children(vec![
            text("Start:"),
            h_space(style::SPACE),
            time_info(now, self.builder.start.clone()),
            h_space(style::DSPACE),
            text("End:"),
            h_space(style::SPACE),
            time_info(now, self.builder.end.clone()),
            h_space(style::DSPACE),
            text("Task:"),
            h_space(style::SPACE),
            task_info(self.builder.task.as_ref(), &self.builder.clipboard_reading),
            h_space(style::DSPACE),
            text("Message:"),
            h_space(style::SPACE),
            text(
                self.builder
                    .msg
                    .as_ref()
                    .map(|e| e.as_ref())
                    .unwrap_or("<no message>"),
            ),
        ]);

        Column::with_children(vec![
            self.top_bar.view(),
            v_space(style::SPACE),
            input.into(),
            v_space(style::SPACE),
            status.into(),
        ])
        .into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::Bs(BookSingleMessage::TextChanged(msg)) => {
                self.update_input(msg);
                self.follow_up_msg()
            }
            Message::ClipboardValue(v) => {
                self.builder.apply_clipboard(v);
                None
            }
            Message::SubmitCurrent(stay_active) => {
                Self::on_submit_message(self.try_build(), &mut self.orig, stay_active)
            }
            Message::StoreSuccess(stay_active) => stay_active.on_main_view_store(),
            _ => self.follow_up_msg(),
        }
    }
}

fn task_info<'a>(v: ParseResult<&'a JiraIssue, &'a ()>, clipboard: &'a ClipRead) -> QElement<'a> {
    match v {
        ParseResult::Valid(t) => task_text(t),
        ParseResult::Invalid(_) => text("invalid"),
        ParseResult::Incomplete => text("incomplete"),
        ParseResult::None => text(clipboard.as_str()),
    }
}

fn task_text(t: &JiraIssue) -> QElement {
    text(&t.ident)
}

#[cfg(test)]
mod test;
