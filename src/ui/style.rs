use iced_core::{Background, Color, Vector};
use iced_native::widget::container::Style;
use iced_native::widget::{button, container};
use iced_winit::Length;

pub const LABEL_WIDTH: Length = Length::Units(40);
pub const TIME_WIDTH: Length = Length::Units(200);
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
            background: Some(Background::Color([0.8, 0.8, 0.95].into())),
            border_radius: 0.0,
            border_width: 2.0,
            border_color: Color::from([0.8, 0.8, 0.95]),
            text_color: Color::BLACK,
        }
    }
}
