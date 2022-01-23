use crate::conf::SettingsRef;
use crate::data::{Action, ActiveDay, RecentIssuesRef};
use crate::ui::book::Book;
use crate::ui::book_single::BookSingleUI;
use crate::ui::current_day::CurrentDayUI;
use crate::ui::export::DayExportUi;
use crate::ui::fast_day_end::FastDayEnd;
use crate::ui::fast_day_start::FastDayStart;
use crate::ui::issue_end_edit::IssueEndEdit;
use crate::ui::issue_start_edit::IssueStartEdit;
use crate::ui::{Exit, ViewId};

pub enum CurrentView {
    Book(Box<Book>),
    Fds(Box<FastDayStart>),
    Fde(Box<FastDayEnd>),
    CdUi(Box<CurrentDayUI>),
    Bs(Box<BookSingleUI>),
    Is(Box<IssueStartEdit>),
    Ie(Box<IssueEndEdit>),
    Export(Box<DayExportUi>),
    Exit(Exit),
}

impl CurrentView {
    pub fn view_id(&self) -> ViewId {
        match &self {
            CurrentView::Book(_) => ViewId::Book,
            CurrentView::Fds(_) => ViewId::FastDayStart,
            CurrentView::Fde(_) => ViewId::FastDayEnd,
            CurrentView::CdUi(_) => ViewId::CurrentDayUi,
            CurrentView::Bs(_) => ViewId::BookSingle,
            CurrentView::Is(_) => ViewId::BookIssueStart,
            CurrentView::Ie(_) => ViewId::BookIssueEnd,
            CurrentView::Export(_) => ViewId::Export,
            CurrentView::Exit(_) => ViewId::Exit,
        }
    }

    pub fn create(
        id: ViewId,
        settings: SettingsRef,
        recent_issues: RecentIssuesRef,
        active_day: Option<&ActiveDay>,
    ) -> CurrentView {
        let guard = settings.load();

        match id {
            ViewId::Book => CurrentView::Book(Book::new(&guard)),
            ViewId::FastDayStart => {
                CurrentView::Fds(FastDayStart::for_work_day(settings, active_day))
            }
            ViewId::FastDayEnd => CurrentView::Fde(FastDayEnd::for_work_day(settings, active_day)),
            ViewId::BookSingle => {
                CurrentView::Bs(BookSingleUI::for_active_day(settings, recent_issues, active_day))
            }
            ViewId::BookIssueStart => {
                CurrentView::Is(IssueStartEdit::for_active_day(settings, active_day))
            }
            ViewId::BookIssueEnd => {
                CurrentView::Ie(IssueEndEdit::for_active_day(settings, active_day))
            }
            ViewId::CurrentDayUi => {
                CurrentView::CdUi(CurrentDayUI::for_active_day(settings, active_day))
            }
            ViewId::Export => {
                CurrentView::Export(DayExportUi::for_active_day(settings, active_day))
            }
            ViewId::Exit => CurrentView::Exit(Exit),
        }
    }

    pub fn create_for_edit(
        value: Action,
        settings: SettingsRef,
        recent_issues: RecentIssuesRef,
        active_day: Option<&ActiveDay>,
    ) -> CurrentView {
        match value {
            Action::Work(a) => {
                let mut ui = BookSingleUI::for_active_day(settings, recent_issues, active_day);
                ui.entry_to_edit(a);
                CurrentView::Bs(ui)
            }
            Action::WorkStart(a) => {
                let mut ui = IssueStartEdit::for_active_day(settings, active_day);
                ui.entry_to_edit(a);
                CurrentView::Is(ui)
            }
            Action::WorkEnd(a) => {
                let mut ui = IssueEndEdit::for_active_day(settings, active_day);
                ui.entry_to_edit(a);
                CurrentView::Ie(ui)
            }
            Action::DayStart(a) => {
                let mut ui = FastDayStart::for_work_day(settings, active_day);
                ui.entry_to_edit(a);
                CurrentView::Fds(ui)
            }
            Action::DayEnd(a) => {
                let mut ui = FastDayEnd::for_work_day(settings, active_day);
                ui.entry_to_edit(a);
                CurrentView::Fde(ui)
            }
            _ => CurrentView::create(ViewId::CurrentDayUi, settings, recent_issues, active_day),
        }
    }
}