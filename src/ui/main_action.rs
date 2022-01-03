use crate::conf::Settings;
use crate::data::ActiveDay;
use crate::db::DB;
use arc_swap::ArcSwap;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct MainAction {
    pub settings: Rc<ArcSwap<Settings>>,
    pub initial_action: InitialAction,
    pub db: DB,
    pub work_day: Rc<RefCell<ActiveDay>>,
}

#[derive(Debug)]
#[allow(dead_code)]
pub enum InitialAction {
    Book,
    Show,
    FastStartDay,
    FastEndDay,
    BookSingle,
    IssueStart,
    IssueEnd,
    PrintDay,
}

impl Default for InitialAction {
    fn default() -> Self {
        InitialAction::Book
    }
}
