use crate::data::Action;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ViewId {
    CurrentDayUi,
    BookSingle,
    BookIssueStart,
    BookIssueEnd,
    FastDayStart,
    FastDayEnd,
    Book,
    Export,
    Exit,
}

impl ViewId {
    pub fn show_recent(self) -> bool {
        matches!(
            self,
            ViewId::BookSingle | ViewId::BookIssueStart | ViewId::BookIssueEnd
        )
    }

    pub fn from_action(action: &Action) -> ViewId {
        match action {
            Action::Work(_) => ViewId::BookSingle,
            Action::WorkEvent(_) => ViewId::Exit,
            Action::WorkStart(_) => ViewId::BookIssueStart,
            Action::WorkEnd(_) => ViewId::BookIssueEnd,
            Action::DayStart(_) => ViewId::FastDayStart,
            Action::DayEnd(_) => ViewId::FastDayEnd,
            Action::DayOff => ViewId::Exit,
            Action::ZA(_) => ViewId::Exit,
            Action::Vacation => ViewId::Exit,
            Action::Sick => ViewId::Exit,
            Action::Doctor(_) => ViewId::Exit,
        }
    }
}
