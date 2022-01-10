use iced_core::{Background, Color, Font, Vector};
use iced_native::widget::container::Style;
use iced_native::widget::{button, container, Button};
use iced_winit::Length;

pub const LABEL_WIDTH: Length = Length::Units(28);
pub const TIME_WIDTH: Length = Length::Units(190);
pub const DESCRIPTION_WIDTH: Length = Length::Units(500);
pub const SPACE_PX: u16 = 10;
pub const SPACE: Length = Length::Units(SPACE_PX);
pub const DSPACE: Length = Length::Units(2 * SPACE_PX);
pub const MIN_WIDGET_WIDTH: Length = Length::Units(200);
pub const WINDOW_PADDING: u16 = 10;
pub const TAB_SPACE: Length = Length::Units(3);

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

pub struct ActiveTab;

impl button::StyleSheet for ActiveTab {
    fn active(&self) -> button::Style {
        button::Style {
            shadow_offset: Vector::new(0.0, 0.0),
            background: Some(Background::Color([0.95, 0.95, 1.0].into())),
            border_radius: 0.0,
            border_width: 2.0,
            border_color: Color::from([0.95, 0.95, 1.0]),
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
