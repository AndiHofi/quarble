#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ViewId {
    CurrentDayUi,
    BookSingle,
    BookIssueStart,
    BookIssueEnd,
    FastDayStart,
    FastDayEnd,
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
}
