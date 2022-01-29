use std::sync::Arc;

use iced_native::keyboard::{KeyCode, Modifiers};
use iced_native::{event, Event};

use crate::data::WeekDayForwarder;
use crate::ui::stay_active::StayActive;
use crate::ui::{Message, ViewId};

pub(crate) fn global_keyboard_handler(
    event: Event,
    status: iced_winit::event::Status,
) -> Option<Message> {
    if let event::Status::Captured = status {
        if let Event::Keyboard(kb) = event {
            handle_control_keyboard_event(kb)
        } else {
            None
        }
    } else if let Event::Keyboard(kb) = event {
        handle_keyboard_event(kb)
    } else {
        None
    }
}

fn handle_control_keyboard_event(key_event: iced_winit::keyboard::Event) -> Option<Message> {
    use iced_winit::keyboard::Event::*;
    match key_event {
        KeyPressed {
            key_code,
            modifiers,
        } => {
            if modifiers.is_empty() {
                match key_code {
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        Some(Message::SubmitCurrent(StayActive::Default))
                    }
                    KeyCode::Escape => Some(Message::Exit),
                    _ => None,
                }
            } else if modifiers.control() && modifiers.shift() {
                if matches!(key_code, KeyCode::Tab) {
                    Some(Message::PrevTab)
                } else {
                    None
                }
            } else if modifiers == Modifiers::CTRL {
                handle_control_shortcuts(key_code)
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Global shortcuts with pressed CTRL key
fn handle_control_shortcuts(key_code: KeyCode) -> Option<Message> {
    match key_code {
        KeyCode::D => Some(Message::RequestDayChange),
        KeyCode::I => Some(Message::ChangeView(ViewId::BookSingle)),
        KeyCode::O => Some(Message::ChangeView(ViewId::FastDayStart)),
        KeyCode::L => Some(Message::ChangeView(ViewId::FastDayEnd)),
        KeyCode::S => Some(Message::ChangeView(ViewId::BookIssueStart)),
        KeyCode::E => Some(Message::ChangeView(ViewId::BookIssueEnd)),
        KeyCode::X => Some(Message::ChangeView(ViewId::Export)),
        KeyCode::C => Some(Message::CopyValue),
        KeyCode::Key1 => Some(Message::ChangeView(ViewId::CurrentDayUi)),
        KeyCode::Enter | KeyCode::NumpadEnter => Some(Message::SubmitCurrent(StayActive::Yes)),
        KeyCode::Left => Some(Message::ChangeDayRelative(-1, Arc::new(WeekDayForwarder))),
        KeyCode::Right => Some(Message::ChangeDayRelative(1, Arc::new(WeekDayForwarder))),
        KeyCode::Tab => Some(Message::NextTab),
        _ => None,
    }
}

fn handle_keyboard_event(key_event: iced_winit::keyboard::Event) -> Option<Message> {
    use iced_winit::keyboard::Event::*;
    match key_event {
        KeyPressed {
            key_code,
            modifiers,
        } => {
            if modifiers.is_empty() {
                match key_code {
                    KeyCode::Escape => Some(Message::Exit),
                    KeyCode::Tab => Some(Message::Next),
                    KeyCode::I => Some(Message::ChangeView(ViewId::BookSingle)),
                    KeyCode::O => Some(Message::ChangeView(ViewId::FastDayStart)),
                    KeyCode::L => Some(Message::ChangeView(ViewId::FastDayEnd)),
                    KeyCode::S => Some(Message::ChangeView(ViewId::BookIssueStart)),
                    KeyCode::E => Some(Message::ChangeView(ViewId::BookIssueEnd)),
                    KeyCode::X => Some(Message::ChangeView(ViewId::Export)),
                    KeyCode::Key1 => Some(Message::ChangeView(ViewId::CurrentDayUi)),
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        Some(Message::SubmitCurrent(StayActive::Default))
                    }
                    KeyCode::Up => Some(Message::Up),
                    KeyCode::Down => Some(Message::Down),
                    KeyCode::Delete => Some(Message::Del),
                    _ => None,
                }
            } else if modifiers == Modifiers::SHIFT | Modifiers::CTRL {
                match key_code {
                    KeyCode::Tab => Some(Message::PrevTab),
                    _ => None,
                }
            } else if modifiers == Modifiers::SHIFT {
                match key_code {
                    KeyCode::Tab => Some(Message::Previous),
                    _ => None,
                }
            } else if modifiers == Modifiers::CTRL {
                match key_code {
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        Some(Message::SubmitCurrent(StayActive::Yes))
                    }
                    key_code => handle_control_shortcuts(key_code),
                }
            } else {
                None
            }
        }
        _ => None,
    }
}
