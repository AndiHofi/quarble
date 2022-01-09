use std::sync::Arc;
use crate::data::{Action, Day};
use crate::ui::{StayActive, ViewId};
use crate::ui::book_single::BookSingleMessage;
use crate::ui::current_day::CurrentDayMessage;
use crate::ui::export::DayExportMessage;
use crate::ui::fast_day_end::FastDayEndMessage;
use crate::ui::fast_day_start::FastDayStartMessage;
use crate::ui::issue_end_edit::IssueEndMessage;
use crate::ui::issue_start_edit::IssueStartMessage;

#[derive(Debug, Clone)]
pub enum Message {
    Update,
    Exit,
    StartInsert,
    Edit,
    Next,
    Previous,
    Up,
    Down,
    CopyValue,
    RequestDayChange,
    ReadClipboard,
    WriteClipboard(Arc<String>),
    ChangeView(ViewId),
    RefreshView,
    Reset,
    SubmitCurrent(StayActive),
    ChangeDay(Day),
    ClipboardValue(Option<String>),
    UpdateCloseOnSafe(bool),
    UpdateStart {
        id: usize,
        input: String,
        valid: bool,
    },
    UpdateEnd {
        id: usize,
        input: String,
        valid: bool,
    },
    UpdateDescription {
        id: usize,
        input: String,
    },
    Export(DayExportMessage),
    Fds(FastDayStartMessage),
    Fde(FastDayEndMessage),
    Bs(BookSingleMessage),
    Is(IssueStartMessage),
    Ie(IssueEndMessage),
    Cd(CurrentDayMessage),
    EditAction(EditAction),
    StoreAction(StayActive, Action),
    ModifyAction {
        stay_active: StayActive,
        orig: Box<Action>,
        update: Box<Action>,
    },
    StoreSuccess(StayActive),
    Error(String),
}

impl Default for Message {
    fn default() -> Self {
        Message::Update
    }
}

#[derive(Clone, Debug)]
pub struct EditAction(pub Box<Action>);

#[derive(Clone, Debug)]
struct ModifyAction {
    stay_active: StayActive,
    orig: Box<Action>,
    update: Box<Action>,
}

#[cfg(test)]
mod test {
    use super::*;
    use std::mem::size_of;

    enum X {
        MA(ModifyAction),
        Something(String),
        Else(Arc<String>),
    }

    #[test]
    fn test() {
        eprintln!("Size of StayActive: {}", size_of::<StayActive>());
        eprintln!("Size of ModifyAction: {}", size_of::<ModifyAction>());
        eprintln!("Size of action: {}", size_of::<Message>());
        eprintln!("Size of X: {}", size_of::<X>());
        eprintln!("Size of Day: {}", size_of::<Day>())
    }
}