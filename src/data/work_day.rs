use crate::data::{Day, Work};

/// Normalized work day
pub struct WorkDay {
    pub date: Day,
    pub entries: Vec<Work>,
    pub synchronized: bool,
}