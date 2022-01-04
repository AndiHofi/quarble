use crate::data::ActiveDay;
use crate::db::DB;
use crate::ui::{SettingsRef, ViewId};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct MainAction {
    pub settings: SettingsRef,
    pub initial_action: ViewId,
    pub db: DB,
    pub work_day: Rc<RefCell<ActiveDay>>,
}

#[derive(Debug)]
pub enum InitialAction {
    Ui(ViewId),
    Cmd(CmdId),
}

#[derive(Clone, Debug)]
pub enum CmdId {
    PrintDay,
}

impl Default for InitialAction {
    fn default() -> Self {
        InitialAction::Ui(ViewId::CurrentDayUi)
    }
}
