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
