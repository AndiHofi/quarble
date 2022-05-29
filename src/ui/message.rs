use std::sync::Arc;

use crate::data::{Action, Day, DayForwarder};
use crate::ui::book_single::BookSingleMessage;
use crate::ui::current_day::CurrentDayMessage;
use crate::ui::export::DayExportMessage;
use crate::ui::fast_day_end::FastDayEndMessage;
use crate::ui::fast_day_start::FastDayStartMessage;
use crate::ui::issue_end_edit::IssueEndMessage;
use crate::ui::issue_start_edit::IssueStartMessage;
use crate::ui::settings_ui::SettingsUIMessage;
use crate::ui::stay_active::StayActive;
use crate::ui::ViewId;

#[derive(Debug, Clone)]
pub enum Message {
    Update,
    Exit,
    Next,
    Previous,
    NextTab,
    PrevTab,
    Up,
    Down,
    Del,
    CopyValue,
    RequestDayChange,
    ReadClipboard,
    WriteClipboard(Arc<String>),
    ChangeView(ViewId),
    RefreshView,
    Reset,
    SubmitCurrent(StayActive),
    ChangeDay(Day),
    ChangeDayRelative(i64, Arc<dyn DayForwarder>),
    ClipboardValue(Option<String>),
    IssueInput(String),
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
    SettingsUi(SettingsUIMessage),
    EditAction(EditAction),
    DeleteAction(DeleteAction),
    StoreAction(StayActive, Action),
    ModifyAction {
        stay_active: StayActive,
        orig: Box<Action>,
        update: Box<Action>,
    },
    StoreSuccess(StayActive),
    Error(String),
    TextChanged(String),
    FilterRecent(Box<str>, Box<str>),
}

impl Default for Message {
    fn default() -> Self {
        Message::Update
    }
}

#[derive(Clone, Debug)]
pub struct EditAction(pub Box<Action>);

#[derive(Clone, Debug)]
pub struct DeleteAction(pub StayActive, pub Box<Action>);
