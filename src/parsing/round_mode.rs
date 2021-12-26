
#[derive(Copy, Clone, Debug)]
pub enum RoundMode {
    None,
    SatUp,
    Up,
    Down,
    SatDown,
    Normal,
}

impl RoundMode {
    pub(crate) fn is_sat(&self) -> bool {
        match self {
            RoundMode::SatUp | RoundMode::SatDown => true,
            _ => false,
        }
    }
}
