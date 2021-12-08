use std::cell::RefCell;
use std::rc::Rc;
use arc_swap::ArcSwap;
use crate::conf::Settings;
use crate::data::WorkDay;
use crate::db::DB;

#[derive(Debug)]
pub struct MainAction {
    pub settings: Rc<ArcSwap<Settings>>,
    pub initial_action: InitialAction,
    pub db: DB,
    pub work_day: Rc<RefCell<WorkDay>>,
}



#[derive(Debug)]
pub enum InitialAction {
    Book,
    Show,
}

impl Default for InitialAction {
    fn default() -> Self {
        InitialAction::Book
    }
}