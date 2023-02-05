use iced_native::widget::Text;
use crate::ui::{MainView, Message, QElement};

pub struct Exit;

impl MainView for Exit {
    fn view(&self) -> QElement {
        Text::new("exiting ...").into()
    }

    fn update(&mut self, _msg: Message) -> Option<Message> {
        Some(Message::ForceExit)
    }
}
