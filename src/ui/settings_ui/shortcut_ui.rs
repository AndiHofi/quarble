use crate::data::JiraIssue;
use crate::ui::my_text_input::MyTextInput;
use crate::ui::style::container_style;
use crate::ui::util::h_space;
use crate::ui::{settings_ui, style, text, QElement};
use iced_core::Length;
use iced_native::widget::{Container, Row, Text};

pub struct ShortCutUi {
    pub shortcut: MyTextInput,
    pub id: MyTextInput,
    pub description: MyTextInput,
    pub default_action: MyTextInput,
}

impl ShortCutUi {
    const SHORTCUT_WIDTH: Length = Length::Units(30);
    const ID_WIDTH: Length = Length::Units(100);
    const DESCRIPTION_WIDTH: Length = Length::Fill;
    const DEFAULT_ACTION_WIDTH: Length = Length::Units(300);

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
            self.shortcut.show_text_input(Self::SHORTCUT_WIDTH).into(),
            h_space(style::SPACE),
            self.id.show_text_input(Self::ID_WIDTH).into(),
            h_space(style::SPACE),
            self.description
                .show_text_input(Self::DESCRIPTION_WIDTH)
                .into(),
            h_space(style::SPACE),
            self.default_action
                .show_text_input(Self::DEFAULT_ACTION_WIDTH)
                .into(),
        ])
        .into()
    }

    pub fn show_header<'a>() -> QElement<'a> {
        fn h<'a>(text: &'static str, width: Length) -> QElement<'a> {
            let mut result = Container::new(Text::new(text).width(width))
                .style(container_style(style::TableHeaderStyle))
                .padding([2, 5]);

            if width == Length::Fill {
                result = result.width(Length::Fill);
            }

            result.into()
        }

        Row::with_children(vec![
            h("Key", Self::SHORTCUT_WIDTH),
            h("Issue number", Self::ID_WIDTH),
            h("Description", Self::DESCRIPTION_WIDTH),
            h("Default action", Self::DEFAULT_ACTION_WIDTH),
        ])
        .spacing(style::SPACE_PX)
        .into()
    }
}
