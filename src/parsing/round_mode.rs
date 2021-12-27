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
        matches!(self, RoundMode::SatUp | RoundMode::SatDown)
    }
}
