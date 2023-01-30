use crate::data::JiraIssue;
use crate::ui::my_text_input::MyTextInput;
use crate::ui::util::h_space;
use crate::ui::{settings_ui, style, QElement};
use iced_core::Length;
use iced_native::widget::Row;

pub struct ShortCutUi {
    pub shortcut: MyTextInput,
    pub id: MyTextInput,
    pub description: MyTextInput,
    pub default_action: MyTextInput,
}

impl ShortCutUi {
    pub fn new(sc: &char, i: &JiraIssue) -> Self {
        Self::build(Some(sc), Some(i))
    }

    pub fn empty() -> Self {
        Self::build(None, None)
    }

    pub fn build(sc: Option<&char>, i: Option<&JiraIssue>) -> Self {
        ShortCutUi {
            shortcut: MyTextInput::new_opt(sc, settings_ui::accept_shortcut),
            id: MyTextInput::new_opt(i.map(|i| i.ident.as_str()), settings_ui::accept_issue_id),
            description: MyTextInput::new_opt(
                i.and_then(|i| i.description.as_deref()),
                settings_ui::no_check,
            ),
            default_action: MyTextInput::new_opt(
                i.and_then(|i| i.default_action.as_deref()),
                settings_ui::no_check,
            ),
        }
    }

    pub fn show(&self) -> QElement {
        Row::with_children(vec![
            self.shortcut.show_text_input(Length::Units(30)).into(),
            h_space(style::SPACE),
            self.id.show_text_input(Length::Units(100)).into(),
            h_space(style::SPACE),
            self.description.show_text_input(Length::Fill).into(),
            h_space(style::SPACE),
            self.default_action
                .show_text_input(Length::Units(300))
                .into(),
        ])
        .into()
    }
}
