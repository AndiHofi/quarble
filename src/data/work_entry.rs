use crate::data::{CurrentWork, Work};

#[derive(Debug, Clone)]
pub enum WorkEntry {
    Work(Work),
    Current(CurrentWork),
}

impl From<Work> for WorkEntry {
    fn from(w: Work) -> Self {
        WorkEntry::Work(w)
    }
}

impl From<CurrentWork> for WorkEntry {
    fn from(w: CurrentWork) -> Self {
        WorkEntry::Current(w)
    }
}


