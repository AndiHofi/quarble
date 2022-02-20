use std::ops::DerefMut;
use crate::ui::Message;
use iced_native::widget::text_input;

pub trait FocusHandler<'a, F>
where
    F: DerefMut<Target=[&'a mut text_input::State]> + IntoIterator<Item = &'a mut text_input::State>,
{
    fn focus_order(&'a mut self) -> F;

    fn has_focus(&'a mut self) -> bool {
        let order = self.focus_order();
        order.into_iter().any(|e| e.is_focused())
    }

    fn remove_focus(&'a mut self) {
        let order = self.focus_order();
        for e in order {
            e.unfocus();
        }
    }

    fn focus_next(&'a mut self) -> Option<Message> {
        let mut order = self.focus_order();
        focus_next(order.deref_mut(), Self::rotate())
    }

    fn focus_previous(&'a mut self) -> Option<Message> {
        let mut order = self.focus_order();
        focus_previous(order.deref_mut(), Self::rotate())
    }

    fn rotate() -> bool {
        true
    }
}

pub(in crate::ui) fn focus_next(
    items: &mut [&mut text_input::State],
    rotate: bool,
) -> Option<Message> {
    if items.is_empty() {
        return None;
    }

    for w in 0..items.len() - 1 {
        if items[w].is_focused() {
            items[w].unfocus();
            items[w + 1].focus();
            items[w + 1].select_all();
            return None;
        }
    }

    let is_last = {
        let last = items.last_mut().unwrap();
        if last.is_focused() {
            last.unfocus();
            true
        } else {
            false
        }
    };

    if is_last {
        return if rotate {
            let first = items.first_mut().unwrap();
            first.focus();
            first.select_all();
            None
        } else {
            Some(Message::Down)
        };
    }

    None
}


pub(in crate::ui) fn focus_previous(
    items: &mut [&mut text_input::State],
    rotate: bool,
) -> Option<Message> {
    if items.is_empty() {
        return None;
    }

    let on_first: bool = {
        let first = items.first_mut().unwrap();
        if first.is_focused() {
            first.unfocus();
            true
        } else {
            false
        }
    };

    if on_first {
        return if rotate {
            let last = items.last_mut().unwrap();
            last.focus();
            last.select_all();
            None
        } else {
            Some(Message::Up)
        };
    }

    for w in 1..items.len() {
        if items[w].is_focused() {
            items[w].unfocus();
            items[w - 1].focus();
            items[w - 1].select_all();
            return None;
        }
    }

    None
}
