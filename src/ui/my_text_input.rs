use crate::ui::util::h_space;
use crate::ui::{style, text, Message, QElement, QRenderer};
use chrono::format::format;
use iced_core::Length;
use iced_native::theme;
use iced_native::widget::text_input::{Action, Id};
use iced_native::widget::{text_input, Row, TextInput};
use std::fmt::{Debug, Formatter};

pub struct MyTextInput {
    pub id: Id,
    pub text: String,
    pub placeholder: String,
    pub accept_input_fn: Box<dyn Fn(&str) -> (bool, Option<Message>)>,
    pub focus_lost_fn: Box<dyn Fn(&mut MyTextInput) -> Option<Message>>,
    pub error: Option<String>,
}

impl Debug for MyTextInput {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MyTextInput(id={:?}, text={}, placeholder={}, error={:?}",
            self.id, self.text, self.placeholder, self.error
        )
    }
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
        Self::msg_aware(text.map(|s| s.to_string()).unwrap_or_default(), move |i| {
            (accept(i), None)
        })
    }

    /// New text-input with input accept function
    ///
    /// The function is called whenever trying to change the text of the text field.
    /// Function parameter is the value of the updated text.
    ///
    /// Function return is a tuple consisting of:
    /// * a boolean, true when the updated text is accepted
    /// * an optional Message to send - this can be used to change focus on specific inputs
    pub fn msg_aware<S, F>(text: S, accept: F) -> Self
    where
        S: ToString,
        F: Fn(&str) -> (bool, Option<Message>) + 'static,
    {
        Self {
            id: Id::unique(),
            text: text.to_string(),
            placeholder: String::new(),
            accept_input_fn: Box::new(accept),
            focus_lost_fn: Box::new(dummy_focus_lost),
            error: None,
        }
    }

    pub fn focus_lost<F>(mut self, handler: F) -> Self
    where
        F: Fn(&mut MyTextInput) -> Option<Message> + 'static,
    {
        self.focus_lost_fn = Box::new(handler);
        self
    }

    pub fn with_placeholder(mut self, placeholder: &str) -> Self {
        self.placeholder = placeholder.to_string();
        self
    }

    pub fn show(&self, label: &'static str) -> QElement {
        self.show_with_input_width(label, Length::Fill)
    }

    pub fn show_with_input_width(&self, label: &'static str, width: Length) -> QElement {
        let mut result: Row<Message, QRenderer> = Row::new();
        result = result.push(text(label));
        result = result.push(h_space(style::SPACE));
        result = result.push(self.show_text_input(width));

        result.into()
    }

    pub fn show_text_input(&self, width: Length) -> TextInput<Message, QRenderer> {
        TextInput::new(&self.placeholder, &self.text, Message::input(&self.id))
            .id(self.id.clone())
            .padding(style::TEXT_INPUT_PADDING)
            .size(style::FONT_SIZE)
            .style(theme::TextInput::Custom(Box::new(style::TextInput {
                error: self.error.is_some(),
            })))
            .on_action(focus_handler(self))
            .width(width)
    }

    pub fn is_focused(&self, focused: &Option<text_input::Id>) -> bool {
        if let Some(id) = focused {
            self.id == *id
        } else {
            false
        }
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

fn focus_handler<'a>(input: &'a MyTextInput) -> impl Fn(Action) -> Option<Message> + 'a {
    |ah| match ah {
        Action::FocusGained => Some(Message::Focus(input.id.clone())),
        _ => None,
    }
}

fn dummy_focus_lost(_input: &mut MyTextInput) -> Option<Message> {
    None
}
