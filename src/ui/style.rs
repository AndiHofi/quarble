use iced_core::Color;
use iced_native::widget::container;
use iced_native::widget::container::Style;
use iced_winit::Length;

pub const LABEL_WIDTH: Length = Length::Units(40);
pub const TIME_WIDTH: Length = Length::Units(200);
pub const DESCRIPTION_WIDTH: Length = Length::Units(500);
pub const SPACE_PX: u16 = 10;
pub const SPACE: Length = Length::Units(SPACE_PX);
pub const DSPACE: Length = Length::Units(2 * SPACE_PX);
pub const MIN_WIDGET_WIDTH: Length = Length::Units(200);
pub const WINDOW_PADDING: u16 = 10;

pub struct ContentStyle;

impl container::StyleSheet for ContentStyle {
    fn style(&self) -> Style {
        let mut s = Style::default();
        s.border_color = Color::BLACK;
        s.border_radius = 2.0;
        s.border_width = 1.0;
        s
    }
}
