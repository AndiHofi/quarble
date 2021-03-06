use iced_core::{Background, Color, Font, Vector};
use iced_native::widget::container::Style;
use iced_native::widget::{button, container, text_input, Button};
use iced_winit::Length;

pub const LABEL_WIDTH: Length = Length::Units(28);
pub const TIME_WIDTH: Length = Length::Units(190);
pub const DESCRIPTION_WIDTH: Length = Length::Units(500);
pub const SPACE_PX: u16 = 10;
pub const SPACE: Length = Length::Units(SPACE_PX);
pub const DSPACE: Length = Length::Units(2 * SPACE_PX);
#[allow(dead_code)]
pub const MIN_WIDGET_WIDTH: Length = Length::Units(200);
pub const WINDOW_PADDING: u16 = 10;
pub const TAB_SPACE: Length = Length::Units(3);
pub const TEXT_INPUT_PADDING: iced_core::Padding = iced_core::Padding {
    top: 3,
    right: 5,
    bottom: 3,
    left: 5,
};
pub const FONT_SIZE: u16 = 16;

pub const HIGHLIGHT_COLOR: Color = Color::from_rgb(0.95, 0.95, 1.0);
pub const ERROR_COLOR: Color = Color::from_rgb(0.5, 0.0, 0.0);
pub const ERROR_COLOR_FOCUSSED: Color = Color::from_rgb(0.9, 0.0, 0.0);
const MAIN_COLOR: Color = Color {
    r: 0.8,
    g: 0.8,
    b: 0.95,
    a: 1.0,
};

const TEXT_MAIN_COLOR: Color = Color {
    r: 0.16,
    g: 0.16,
    b: 0.19,
    a: 1.0,
};

pub const DEFAULT_BACKGROUND: Background = Background::Color(Color::from_rgb(1.0, 1.0, 1.0));
pub const ODD_BACKGROUND: Background = Background::Color(HIGHLIGHT_COLOR);
pub const SELECTED_BACKGROUND: Background = Background::Color(MAIN_COLOR);

pub struct ContentStyle;

impl container::StyleSheet for ContentStyle {
    fn style(&self) -> Style {
        Style {
            border_color: Color::BLACK,
            border_radius: 2.0,
            border_width: 1.0,
            ..Style::default()
        }
    }
}

pub enum RowState {
    Even,
    Odd,
    Selected,
}

pub struct ContentRow {
    pub state: RowState,
}

impl container::StyleSheet for ContentRow {
    fn style(&self) -> Style {
        let background = match self.state {
            RowState::Even => DEFAULT_BACKGROUND,
            RowState::Odd => ODD_BACKGROUND,
            RowState::Selected => SELECTED_BACKGROUND,
        };
        let background = Some(background);

        Style {
            background,
            ..Style::default()
        }
    }
}

pub struct EditButton;

impl button::StyleSheet for EditButton {
    fn active(&self) -> button::Style {
        button::Style {
            shadow_offset: Vector::new(0.0, 0.0),
            background: Some(Background::Color(MAIN_COLOR)),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: MAIN_COLOR,
            text_color: TEXT_MAIN_COLOR,
        }
    }
}

pub struct TextInput {
    pub error: bool,
}

const DEFAULT_TI_STYLE: text_input::Style = text_input::Style {
    background: Background::Color(Color::WHITE),
    border_radius: 5.0,
    border_width: 1.0,
    border_color: Color::from_rgb(0.7, 0.7, 0.7),
};

impl text_input::StyleSheet for TextInput {
    fn active(&self) -> text_input::Style {
        if self.error {
            text_input::Style {
                border_color: ERROR_COLOR,
                ..DEFAULT_TI_STYLE
            }
        } else {
            text_input::Style {
                border_color: Color::from_rgb(0.7, 0.7, 0.7),
                ..DEFAULT_TI_STYLE
            }
        }
    }

    fn focused(&self) -> text_input::Style {
        if self.error {
            text_input::Style {
                border_color: ERROR_COLOR_FOCUSSED,
                ..DEFAULT_TI_STYLE
            }
        } else {
            text_input::Style {
                border_color: Color::from_rgb(0.5, 0.5, 0.5),
                ..DEFAULT_TI_STYLE
            }
        }
    }

    fn placeholder_color(&self) -> Color {
        Color::from_rgb(0.7, 0.7, 0.7)
    }

    fn value_color(&self) -> Color {
        Color::BLACK
    }

    fn selection_color(&self) -> Color {
        Color::from_rgb(0.8, 0.8, 1.0)
    }
}

pub struct ActiveTab;

impl button::StyleSheet for ActiveTab {
    fn active(&self) -> button::Style {
        button::Style {
            shadow_offset: Vector::new(0.0, 0.0),
            background: Some(Background::Color(HIGHLIGHT_COLOR)),
            border_radius: 0.0,
            border_width: 2.0,
            border_color: HIGHLIGHT_COLOR,
            text_color: Color::BLACK,
        }
    }

    fn hovered(&self) -> button::Style {
        self.active()
    }

    fn pressed(&self) -> button::Style {
        self.active()
    }
}

pub struct Tab;

impl button::StyleSheet for Tab {
    fn active(&self) -> button::Style {
        button::Style {
            shadow_offset: Vector::new(0.0, 0.0),
            background: Some(Background::Color(MAIN_COLOR)),
            border_radius: 0.0,
            border_width: 2.0,
            border_color: MAIN_COLOR,
            text_color: TEXT_MAIN_COLOR,
        }
    }
}

const UBUNTU_BOLD: &[u8] = include_bytes!("../../fonts/Ubuntu-B.ttf");

pub fn button_font() -> Font {
    Font::External {
        name: "Ubuntu Bold",
        bytes: UBUNTU_BOLD,
    }
}

pub fn inline_button<'a>(
    state: &'a mut button::State,
    text: &str,
) -> Button<'a, super::Message, <super::Quarble as iced_winit::Program>::Renderer> {
    Button::new(
        state,
        iced_native::widget::Text::new(text).font(button_font()),
    )
    .style(EditButton)
    .padding([2, 5])
}
