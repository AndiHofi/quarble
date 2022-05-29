use crate::ui::util::h_space;
use crate::ui::{style, text, Message, QElement, QRenderer};
use iced_core::Length;
use iced_native::widget::{text_input, Row, TextInput};

pub struct MyTextInput {
    pub text: String,
    pub input: text_input::State,
    pub accept_input_fn: Box<dyn Fn(&str) -> (bool, Option<Message>)>,
    pub error: Option<String>,
}

impl MyTextInput {
    pub fn new<F>(text: impl ToString, accept: F) -> Self
    where
        F: Fn(&str) -> bool + 'static,
    {
        Self::new_opt(Some(text), accept)
    }

    pub fn new_opt<S, F>(text: Option<S>, accept: F) -> Self
    where
        S: ToString,
        F: Fn(&str) -> bool + 'static,
    {
        Self::msg_aware(text.map(|s|s.to_string()).unwrap_or_default(), move |i| (accept(i), None))
    }

    pub fn msg_aware<S, F>(text: S, accept: F) -> Self
        where
            S: ToString,
            F: Fn(&str) -> (bool, Option<Message>) + 'static,
    {
        Self {
            text: text.to_string(),
            input: text_input::State::new(),
            accept_input_fn: Box::new(accept),
            error: None,
        }
    }


    pub fn show(&mut self, label: &'static str) -> QElement {
        self.show_with_input_width(label, Length::Fill)
    }

    pub fn show_with_input_width(&mut self, label: &'static str, width: Length) -> QElement {
        let mut result: Row<Message, QRenderer> = Row::new();
        result = result.push(text(label));
        result = result.push(h_space(style::SPACE));
        result = result.push(self.show_text_input(width));

        result.into()
    }

    pub fn show_text_input(&mut self, width: Length) -> TextInput<Message, QRenderer> {
        TextInput::new(&mut self.input, "", &self.text, Message::TextChanged)
            .padding(style::TEXT_INPUT_PADDING)
            .size(style::FONT_SIZE)
            .style(style::TextInput {
                error: self.error.is_some(),
            })
            .width(width)
    }

    pub fn is_focused(&self) -> bool {
        self.input.is_focused()
    }

    pub fn accept_input(&mut self, text: String) -> Option<Message> {
        let (accept, msg) = (*self.accept_input_fn)(text.as_str());
        if accept {
            self.text = text;
        }
        msg
    }

    pub fn consume_err<T>(&mut self, result: Result<T, String>) -> Result<T, ()> {
        match result {
            Ok(t) => {
                self.error = None;
                Ok(t)
            }
            Err(e) => {
                self.error = Some(e);
                Err(())
            }
        }
    }
}
