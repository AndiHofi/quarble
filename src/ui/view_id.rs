#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ViewId {
    CurrentDayUi,
    BookSingle,
    BookIssueStart,
    BookIssueEnd,
    FastDayStart,
    FastDayEnd,
    Export,
    Settings,
    Exit,
}

impl ViewId {
    pub const TAB_ORDER: &'static [ViewId] = &[
        Self::CurrentDayUi,
        Self::FastDayStart,
        Self::FastDayEnd,
        Self::BookSingle,
        Self::BookIssueStart,
        Self::BookIssueEnd,
        Self::Export,
        Self::Settings,
    ];

    pub fn show_recent(self) -> bool {
        matches!(
            self,
            ViewId::BookSingle | ViewId::BookIssueStart | ViewId::BookIssueEnd
        )
    }
}
