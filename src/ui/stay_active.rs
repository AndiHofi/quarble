use crate::conf::Settings;
use crate::ui::Message;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StayActive {
    Default,
    Yes,
    No,
}

impl Default for StayActive {
    fn default() -> Self {
        StayActive::Default
    }
}

impl StayActive {
    pub fn close_app(self, settings: &Settings) -> bool {
        match self {
            StayActive::Default => settings.close_on_safe,
            StayActive::No => true,
            StayActive::Yes => false,
        }
    }

    pub fn apply_settings(self, settings: &Settings) -> Self {
        match self {
            StayActive::Default => {
                if settings.close_on_safe {
                    StayActive::No
                } else {
                    StayActive::Yes
                }
            }
            v => v,
        }
    }

    pub fn do_close(self) -> bool {
        matches!(self, StayActive::No)
    }

    pub fn on_main_view_store(self) -> Option<Message> {
        Some(if self.do_close() {
            Message::Exit
        } else {
            Message::Reset
        })
    }
}
