use iced_native::widget::Row;
use iced_wgpu::TextInput;
use iced_winit::widget::{text_input, Column};

use parsing::WorkBuilder;

use crate::conf::{Settings, SettingsRef};
use crate::data::{Action, ActiveDay, JiraIssue, Work};
use crate::parsing::parse_result::ParseResult;
use crate::ui::clip_read::ClipRead;
use crate::ui::top_bar::TopBar;
use crate::ui::util::{h_space, v_space};
use crate::ui::{
    day_info_message, style, text, time_info, MainView, Message, QElement, StayActive,
};

mod parsing;

#[derive(Clone, Debug)]
pub enum BookSingleMessage {
    TextChanged(String),
}

pub struct BookSingleUI {
    top_bar: TopBar,
    input_state: text_input::State,
    input: String,
    data: Option<Work>,
    builder: WorkBuilder,
    settings: SettingsRef,
    orig: Option<Work>,
}

impl BookSingleUI {
    fn parse_input(&mut self, text: &str) {
        self.builder.parse_input(&self.settings.load(), text)
    }

    pub fn for_active_day(settings: SettingsRef, active_day: Option<&ActiveDay>) -> Box<Self> {
        Box::new(Self {
            top_bar: TopBar {
                title: "Book issue:",
                help_text: "(start [end])|duration <issue id> <message>",
                info: day_info_message(active_day),
                settings: settings.clone(),
            },
            input_state: text_input::State::focused(),
            input: "".to_string(),
            data: None,
            builder: Default::default(),
            settings,
            orig: None,
        })
    }

    pub fn entry_to_edit(&mut self, e: Work) {
        let text = format!("{} {} {} {}", e.start, e.end, e.task.ident, e.description);
        self.parse_input(&text);
        self.input = text;
        self.orig = Some(e);
    }

    fn follow_up_msg(&mut self) -> Option<Message> {
        if self.builder.needs_clipboard() {
            self.builder.clipboard_reading = ClipRead::Reading;
            Some(Message::ReadClipboard)
        } else {
            None
        }
    }

    fn on_submit(&mut self, stay_active: StayActive) -> Option<Message> {
        let action = self
            .builder
            .try_build(self.settings.load().timeline.time_now())
            .map(Action::Work);

        if let Some(action) = action {
            if let Some(orig) = std::mem::take(&mut self.orig) {
                Some(Message::ModifyAction {
                    stay_active,
                    orig: Box::new(Action::Work(orig)),
                    update: Box::new(action),
                })
            } else {
                Some(Message::StoreAction(stay_active, action))
            }
        } else {
            None
        }
    }
}

impl MainView for BookSingleUI {
    fn view(&mut self, _settings: &Settings) -> QElement {
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
                self.parse_input(&msg);
                self.input = msg;
                self.follow_up_msg()
            }
            Message::ClipboardValue(v) => {
                self.builder.apply_clipboard(v);
                None
            }
            Message::SubmitCurrent(stay_active) => self.on_submit(stay_active),
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
