use crate::conf::Settings;
use crate::data::{ActiveDay, JiraIssue, Work};
use crate::parsing::parse_result::ParseResult;
use crate::ui::clip_read::ClipRead;
use crate::ui::util::{h_space, v_space};
use crate::ui::{input_message, style, text, time_info, MainView, Message, QElement};
use crate::util::Timeline;
use iced_native::widget::Row;
use iced_wgpu::TextInput;
use iced_winit::widget::{text_input, Column, Text};
use parsing::WorkBuilder;
use std::sync::Arc;

mod parsing;

#[derive(Clone, Debug)]
pub enum BookSingleMessage {
    TextChanged(String),
}

pub struct BookSingleUI {
    input_state: text_input::State,
    input: String,
    data: Option<Work>,
    builder: WorkBuilder,
    input_message: String,
    timeline: Timeline,
    settings: Arc<Settings>,
}

impl BookSingleUI {
    fn parse_input(&mut self, text: &str) {
        self.builder.parse_input(&self.settings, text)
    }

    pub fn for_active_day(settings: Arc<Settings>, active_day: Option<&ActiveDay>) -> Box<Self> {
        let actions = active_day
            .map(|a| a.actions())
            .unwrap_or(ActiveDay::no_action());
        let timeline = settings.timeline.clone();
        Box::new(Self {
            input_state: text_input::State::focused(),
            input: "".to_string(),
            data: None,
            builder: Default::default(),
            input_message: input_message("Book issue", actions),
            timeline,
            settings,
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

    fn on_submit_message(&self, settings: &Settings) -> Message {
        self.builder
            .try_build(settings.timeline.time_now(), settings)
            .map(|e| Message::StoreAction(crate::data::Action::Work(e)))
            .unwrap_or_default()
    }
}

impl MainView for BookSingleUI {
    fn view(&mut self, settings: &Settings) -> QElement {
        let on_submit = self.on_submit_message(settings);

        let msg = Text::new(&self.input_message);
        let input = TextInput::new(&mut self.input_state, "", &self.input, |s| {
            Message::Bs(BookSingleMessage::TextChanged(s))
        })
        .on_submit(on_submit);

        let now = settings.timeline.time_now();

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
            msg.into(),
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
            Message::StoreSuccess => Some(Message::Exit),
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
