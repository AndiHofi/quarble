use crate::ui::style;
use crate::ui::util::h_space;
use crate::ui::{Message, QElement, ViewId};
use iced_core::Length;
use iced_native::theme;
use iced_native::widget::Text;
use iced_winit::widget::{button, Button, Row};

pub struct TabBar {
    active_view: ViewId,
    current_day_ui: button::State,
    fast_day_start: button::State,
    fast_day_end: button::State,
    book_single: button::State,
    book_issue_start: button::State,
    book_issue_end: button::State,
    export: button::State,
    settings: button::State,
    exit: button::State,
}

impl TabBar {
    pub fn new(initial_view: ViewId) -> Self {
        Self {
            active_view: initial_view,

            current_day_ui: Default::default(),
            fast_day_start: Default::default(),
            fast_day_end: Default::default(),
            book_single: Default::default(),
            book_issue_start: Default::default(),
            book_issue_end: Default::default(),
            export: Default::default(),
            settings: Default::default(),
            exit: Default::default(),
        }
    }

    pub fn view(&self) -> QElement {
        let active = self.active_view;

        let buttons: Vec<QElement> = vec![
            h_space(style::TAB_SPACE),
            tab_button(active, "Overview (1)", ViewId::CurrentDayUi),
            h_space(style::TAB_SPACE),
            tab_button(active, "Start work (o)", ViewId::FastDayStart),
            h_space(style::TAB_SPACE),
            tab_button(active, "Stop work (l)", ViewId::FastDayEnd),
            h_space(style::TAB_SPACE),
            tab_button(active, "Book issue (i)", ViewId::BookSingle),
            h_space(style::TAB_SPACE),
            tab_button(active, "Start issue (s)", ViewId::BookIssueStart),
            h_space(style::TAB_SPACE),
            tab_button(active, "End issue (e)", ViewId::BookIssueEnd),
            h_space(style::TAB_SPACE),
            tab_button(active, "Export (x)", ViewId::Export),
            h_space(style::TAB_SPACE),
            tab_button(active, "Settings (t)", ViewId::Settings),
            h_space(Length::Fill),
            tab_button(active, "x", ViewId::Exit),
            h_space(style::TAB_SPACE),
        ];

        Row::with_children(buttons).height(Length::Units(30)).into()
    }

    pub fn select_next(&self) -> Option<ViewId> {
        if let Some((index, _)) = ViewId::TAB_ORDER
            .iter()
            .enumerate()
            .find(|(_, id)| **id == self.active_view)
        {
            ViewId::TAB_ORDER
                .get((index + 1) % ViewId::TAB_ORDER.len())
                .cloned()
        } else {
            None
        }
    }

    pub fn select_previous(&self) -> Option<ViewId> {
        if let Some((index, _)) = ViewId::TAB_ORDER
            .iter()
            .enumerate()
            .find(|(_, id)| **id == self.active_view)
        {
            let new_view = if index == 0 {
                *ViewId::TAB_ORDER.last().unwrap()
            } else {
                ViewId::TAB_ORDER[index - 1]
            };
            Some(new_view)
        } else {
            None
        }
    }

    pub fn set_active_view(&mut self, view: ViewId) {
        self.active_view = view;
    }
}

fn tab_button<'a>(active: ViewId, text: &'static str, v: ViewId) -> QElement<'a> {
    let button =
        Button::new(Text::new(text).font(style::button_font())).on_press(Message::ChangeView(v));
    let style: Box<dyn button::StyleSheet<Style = iced_native::Theme> + 'static> = if v == active {
        Box::new(style::ActiveTab)
    } else {
        Box::new(style::Tab)
    };
    button.style(theme::Button::Custom(style)).into()
}
