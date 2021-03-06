use iced_core::Length;
use std::str::FromStr;

use crate::ui::{Message, QElement};

pub(super) fn valid_start_time(id: usize, min_val: u32, input: String) -> Message {
    match valid_base_time(&input) {
        (true, None) => Message::UpdateStart {
            id,
            input,
            valid: false,
        },
        (true, Some(val)) => Message::UpdateStart {
            id,
            input,
            valid: val >= min_val,
        },
        (false, _) => Message::Update,
    }
}

pub(super) fn valid_end_time(id: usize, min_val: u32, input: String) -> Message {
    match valid_base_time(&input) {
        (true, None) => Message::UpdateEnd {
            id,
            input,
            valid: false,
        },
        (true, Some(val)) => Message::UpdateEnd {
            id,
            input,
            valid: val >= min_val,
        },
        (false, _) => Message::Update,
    }
}

pub(super) fn valid_base_time(input: &str) -> (bool, Option<u32>) {
    if input.is_empty() {
        return (true, None);
    } else if let Some((h, m)) = input.split_once(':') {
        if m.is_empty() {
            return (true, None);
        }
        if let (Ok(h), Ok(m)) = (u32::from_str(h), u32::from_str(m)) {
            if h < 24 && m < 60 {
                return (true, Some(h * 24 + m));
            }
        }
    } else if let Some((h, p)) = input.split_once(&[',', '.'][..]) {
        if p.is_empty() {
            return (true, None);
        }
        if let (Ok(h), Ok(p)) = (u32::from_str(h), u32::from_str(p)) {
            if h < 24 && p < 100 {
                return (true, Some(h * 24 + (p * 60 / 100)));
            }
        }
    } else if let Ok(t) = u32::from_str(input) {
        if t < 24 {
            return (true, Some(t * 24));
        }
    }

    (false, None)
}

pub(in crate::ui) fn v_space<'a>(l: Length) -> QElement<'a> {
    iced_winit::widget::Space::with_height(l).into()
}

pub(in crate::ui) fn h_space<'a>(l: Length) -> QElement<'a> {
    iced_winit::widget::Space::with_width(l).into()
}
