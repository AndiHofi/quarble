use crate::ui::style;
use crate::ui::util::h_space;
use crate::ui::{Message, QElement, ViewId};
use iced_core::Length;
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
            exit: Default::default(),
        }
    }

    pub fn view(&mut self) -> QElement {
        let active = self.active_view;

        let buttons: Vec<QElement> = vec![
            h_space(style::TAB_SPACE),
            tab_button(
                active,
                &mut self.current_day_ui,
                "Overview (1)",
                ViewId::CurrentDayUi,
            ),
            h_space(style::TAB_SPACE),
            tab_button(
                active,
                &mut self.fast_day_start,
                "Start work (o)",
                ViewId::FastDayStart,
            ),
            h_space(style::TAB_SPACE),
            tab_button(
                active,
                &mut self.fast_day_end,
                "Stop work (l)",
                ViewId::FastDayEnd,
            ),
            h_space(style::TAB_SPACE),
            tab_button(
                active,
                &mut self.book_single,
                "Book issue (i)",
                ViewId::BookSingle,
            ),
            h_space(style::TAB_SPACE),
            tab_button(
                active,
                &mut self.book_issue_start,
                "Start issue (s)",
                ViewId::BookIssueStart,
            ),
            h_space(style::TAB_SPACE),
            tab_button(
                active,
                &mut self.book_issue_end,
                "End issue (e)",
                ViewId::BookIssueEnd,
            ),
            h_space(style::TAB_SPACE),
            tab_button(active, &mut self.export, "Export (x)", ViewId::Export),
            h_space(Length::Fill),
            tab_button(active, &mut self.exit, "x", ViewId::Exit),
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
            ViewId::TAB_ORDER.get((index + 1) % ViewId::TAB_ORDER.len()).cloned()
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

fn tab_button<'a>(
    active: ViewId,
    s: &'a mut button::State,
    text: &'static str,
    v: ViewId,
) -> QElement<'a> {
    let button =
        Button::new(s, Text::new(text).font(style::button_font())).on_press(Message::ChangeView(v));
    let style: Box<dyn button::StyleSheet + 'static> = if v == active {
        Box::new(style::ActiveTab)
    } else {
        Box::new(style::Tab)
    };
    button.style(style).into()
}
