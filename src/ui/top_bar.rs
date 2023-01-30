use crate::conf::SettingsRef;
use crate::ui::util::h_space;
use crate::ui::{style, text, Message, QElement};
use iced_core::Length;
use iced_native::widget::Checkbox;
use iced_winit::widget::Row;

#[derive(Debug)]
pub struct TopBar {
    pub title: &'static str,
    pub help_text: &'static str,
    pub info: String,
    pub settings: SettingsRef,
}

impl TopBar {
    pub fn view(&self) -> QElement {
        Row::with_children(vec![
            text(self.title),
            h_space(style::DSPACE),
            text(self.help_text),
            h_space(style::DSPACE),
            text(&self.info),
            h_space(Length::Fill),
            Checkbox::new(
                self.settings.load().close_on_safe,
                "auto close",
                Message::UpdateCloseOnSafe,
            )
            .into(),
        ])
        .into()
    }
}
