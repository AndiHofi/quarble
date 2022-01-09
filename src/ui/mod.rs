use std::collections::BTreeSet;
use std::rc::Rc;

use arc_swap::ArcSwap;
use iced_core::keyboard::{KeyCode, Modifiers};
use iced_core::{Color, Padding};
use iced_native::clipboard;
use iced_wgpu::Text;
use iced_winit::settings::SettingsWindowConfigurator;
use iced_winit::widget::{Column, Container};
use iced_winit::{event, Command, Subscription};
use iced_winit::{Element, Mode};
use iced_winit::{Event, Program};

use crate::conf::{update_settings, SettingsRef};
use crate::data::{Action, ActiveDay, Day, TimedAction};
use crate::db::DB;
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::time_limit::TimeLimit;
use crate::ui::book::Book;
use crate::ui::book_single::BookSingleUI;
use crate::ui::current_day::{CurrentDayMessage, CurrentDayUI};
use crate::ui::export::{DayExportMessage, DayExportUi};
use crate::ui::fast_day_end::FastDayEnd;
use crate::ui::fast_day_start::FastDayStart;
use crate::ui::issue_end_edit::IssueEndEdit;
use crate::ui::issue_start_edit::IssueStartEdit;
use crate::ui::main_action::MainAction;
use crate::ui::window_configurator::{DisplaySelection, MyWindowConfigurator};
use crate::Settings;

use crate::ui::message::EditAction;
use crate::ui::tab_bar::TabBar;
pub use message::Message;
pub use view_id::ViewId;

mod book;
mod book_single;
mod clip_read;
mod current_day;
mod entry_edit;
mod export;
pub mod fast_day_end;
pub mod fast_day_start;
mod issue_end_edit;
mod issue_start_edit;
pub mod main_action;
mod message;
mod style;
mod tab_bar;
mod top_bar;
mod util;
mod view_id;
mod window_configurator;
mod work_entry_edit;
mod work_start_edit;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum StayActive {
    Default,
    Yes,
    No,
}
impl Default for StayActive {
    fn default() -> Self {
        StayActive::Default
    }
}

impl StayActive {
    pub fn close_app(self, settings: &Settings) -> bool {
        match self {
            StayActive::Default => settings.close_on_safe,
            StayActive::No => true,
            StayActive::Yes => false,
        }
    }

    pub fn apply_settings(self, settings: &Settings) -> Self {
        match self {
            StayActive::Default => {
                if settings.close_on_safe {
                    StayActive::No
                } else {
                    StayActive::Yes
                }
            }
            v => v,
        }
    }

    pub fn do_close(self) -> bool {
        matches!(self, StayActive::No)
    }

    pub fn on_main_view_store(self) -> Option<Message> {
        Some(if self.do_close() {
            Message::Exit
        } else {
            Message::Reset
        })
    }
}

pub fn show_ui(main_action: MainAction) -> Rc<ArcSwap<Settings>> {
    let config_settings = main_action.settings.clone();
    let window_configurator = MyWindowConfigurator {
        base: SettingsWindowConfigurator {
            window: Default::default(),
            id: Some("quarble".to_string()),
            mode: Mode::Windowed,
        },
        display_selection: DisplaySelection::Largest,
    };
    let renderer_settings = iced_wgpu::Settings {
        antialiasing: Some(iced_wgpu::settings::Antialiasing::MSAAx4),
        ..iced_wgpu::Settings::from_env()
    };
    iced_winit::application::run_with_window_configurator::<
        Quarble,
        iced_futures::executor::ThreadPool,
        iced_wgpu::window::Compositor,
        _,
    >(main_action, renderer_settings, window_configurator, true)
    .unwrap();

    config_settings
}

pub struct Quarble {
    current_view: CurrentView,
    settings: SettingsRef,
    db: DB,
    active_day: Option<ActiveDay>,
    active_day_dirty: bool,
    initial_view: ViewId,
    tab_bar: TabBar,
}

impl iced_winit::Program for Quarble {
    type Renderer = iced_wgpu::Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        let mut message = Some(message);
        while let Some(current) = message.take() {
            match current {
                Message::Error(msg) => eprintln!("Got an error: {}", msg),
                Message::Exit => {
                    self.tab_bar.set_active_view(ViewId::CurrentDayUi);
                    self.current_view = CurrentView::Exit(Exit);
                }
                Message::UpdateCloseOnSafe(new_value) => update_settings(&self.settings, |s| {
                    s.close_on_safe = new_value;
                }),
                Message::RequestDayChange => {
                    if let CurrentView::CdUi(ui) = &mut self.current_view {
                        message = ui.update(Message::Cd(CurrentDayMessage::StartDayChange))
                    } else {
                        self.tab_bar.set_active_view(ViewId::CurrentDayUi);
                        self.current_view = CurrentView::create(
                            ViewId::CurrentDayUi,
                            self.settings.clone(),
                            self.active_day.as_ref(),
                        );
                        message = Some(Message::Cd(CurrentDayMessage::StartDayChange));
                    }
                }
                Message::ChangeDay(day) => match self.db.get_day(day) {
                    Ok(day) => {
                        self.active_day = Some(day);
                        message = Some(Message::RefreshView);
                    }
                    Err(e) => {
                        message = Some(Message::Error(format!("{:?}", e)));
                        self.active_day = None;
                    }
                },
                Message::ChangeView(view_id) => {
                    if self.current_view.view_id() != view_id {
                        self.tab_bar.set_active_view(view_id);
                        self.current_view = CurrentView::create(
                            view_id,
                            self.settings.clone(),
                            self.active_day.as_ref(),
                        );
                    }
                }
                Message::RefreshView => {
                    self.tab_bar.set_active_view(self.current_view.view_id());
                    self.current_view = CurrentView::create(
                        self.current_view.view_id(),
                        self.settings.clone(),
                        self.active_day.as_ref(),
                    );
                }
                Message::Reset => {
                    message = Some(Message::ChangeView(self.initial_view));
                }
                Message::EditAction(EditAction(action)) => {
                    self.current_view = CurrentView::create_for_edit(
                        *action,
                        self.settings.clone(),
                        self.active_day.as_ref(),
                    );
                    self.tab_bar.set_active_view(self.current_view.view_id());
                }
                Message::StoreAction(stay_active, action) => {
                    if let Some(ref mut active_day) = self.active_day {
                        active_day.add_action(action);
                        message = match self.db.store_day(active_day.get_day(), active_day) {
                            Ok(()) => Some(Message::StoreSuccess(
                                stay_active.apply_settings(&self.settings.load()),
                            )),
                            Err(e) => Some(Message::Error(format!("{:?}", e))),
                        };
                    }
                }
                Message::ModifyAction {
                    stay_active,
                    orig,
                    update,
                } => {
                    if let Some(ref mut active_day) = self.active_day {
                        let actions = active_day.actions_mut();
                        if actions.remove(&orig) {
                            actions.insert(*update);
                            message = Some(Message::StoreSuccess(
                                stay_active.apply_settings(&self.settings.load()),
                            ));
                        } else {
                            message = Some(Message::Error(
                                "Could not update action. Did not find original".to_string(),
                            ));
                        }
                    }
                }
                Message::CopyValue => match self.current_view.view_id() {
                    ViewId::Export => {
                        message = Some(Message::Export(DayExportMessage::TriggerExport));
                    }
                    ViewId::CurrentDayUi => {
                        self.tab_bar.set_active_view(ViewId::Export);
                        self.current_view = CurrentView::create(
                            ViewId::Export,
                            self.settings.clone(),
                            self.active_day.as_ref(),
                        );
                        message = Some(Message::Export(DayExportMessage::TriggerExport));
                    }
                    _ => (),
                },
                Message::ReadClipboard => {
                    let clipboard = iced_native::command::Action::Clipboard(
                        clipboard::Action::Read(Box::new(Message::ClipboardValue)),
                    );
                    return Command::single(clipboard);
                }
                Message::WriteClipboard(value) => {
                    let clipboard = iced_native::command::Action::Clipboard(
                        clipboard::Action::Write(value.to_string()),
                    );
                    return Command::single(clipboard);
                }
                m => match &mut self.current_view {
                    CurrentView::Book(b) => {
                        message = b.update(m);
                    }
                    CurrentView::Fds(fds) => {
                        message = fds.update(m);
                    }
                    CurrentView::Fde(fde) => {
                        message = fde.update(m);
                    }
                    CurrentView::Bs(bs) => {
                        message = bs.update(m);
                    }
                    CurrentView::Is(is) => {
                        message = is.update(m);
                    }
                    CurrentView::Ie(ie) => {
                        message = ie.update(m);
                    }
                    CurrentView::CdUi(cd) => {
                        message = cd.update(m);
                    }
                    CurrentView::Export(ex) => {
                        message = ex.update(m);
                    }
                    _ => {}
                },
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message, Self::Renderer> {
        let settings = self.settings.load();
        let content = match &mut self.current_view {
            CurrentView::Book(book) => book.view(&settings),
            CurrentView::Fds(fds) => fds.view(&settings),
            CurrentView::Fde(fde) => fde.view(&settings),
            CurrentView::CdUi(cdui) => cdui.view(&settings),
            CurrentView::Bs(bs) => bs.view(&settings),
            CurrentView::Exit(exit) => exit.view(&settings),
            CurrentView::Is(is) => is.view(&settings),
            CurrentView::Ie(ie) => ie.view(&settings),
            CurrentView::Export(ex) => ex.view(&settings),
        };
        let element = Container::new(content)
            .padding(Padding::new(style::WINDOW_PADDING))
            .into();
        let main: QElement = Column::with_children(vec![self.tab_bar.view(), element]).into();

        if self.settings.load().debug {
            main.explain(Color::new(0.5, 0.5, 0.5, 0.5))
        } else {
            main
        }
    }
}

impl iced_winit::Application for Quarble {
    type Flags = MainAction;

    fn new(flags: MainAction) -> (Self, Command<Message>) {
        let db = flags.db;

        let settings = flags.settings;
        let active_day = db.get_day(Day::today()).map(Option::from);
        let (initial_message, active_day) = match active_day {
            Ok(active_day) => (None, active_day),
            Err(e) => (Some(Message::Error(format!("{:?}", e))), None),
        };

        let current_view =
            CurrentView::create(flags.initial_view, settings.clone(), active_day.as_ref());

        let mut quarble = Quarble {
            current_view,
            settings,
            db,
            active_day,
            active_day_dirty: false,
            initial_view: flags.initial_view,
            tab_bar: TabBar::new(flags.initial_view),
        };

        let command = if let Some(initial_message) = initial_message {
            quarble.update(initial_message)
        } else {
            Command::none()
        };

        (quarble, command)
    }

    fn title(&self) -> String {
        "Quarble".to_string()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_winit::subscription::events_with(global_keyboard_handler)
    }

    fn should_exit(&self) -> bool {
        matches!(&self.current_view, CurrentView::Exit(_))
    }
}

fn global_keyboard_handler(event: Event, status: iced_winit::event::Status) -> Option<Message> {
    if let event::Status::Captured = status {
        if let Event::Keyboard(kb) = event {
            handle_control_keyboard_event(kb)
        } else {
            None
        }
    } else if let Event::Keyboard(kb) = event {
        handle_keyboard_event(kb)
    } else {
        None
    }
}

fn handle_control_keyboard_event(key_event: iced_winit::keyboard::Event) -> Option<Message> {
    use iced_winit::keyboard::Event::*;
    match key_event {
        KeyPressed {
            key_code,
            modifiers,
        } => {
            if modifiers.is_empty() {
                match key_code {
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        Some(Message::SubmitCurrent(StayActive::Default))
                    }
                    KeyCode::Escape => Some(Message::Exit),
                    _ => None,
                }
            } else if modifiers == Modifiers::CTRL {
                handle_control_shortcuts(key_code)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn handle_control_shortcuts(key_code: KeyCode) -> Option<Message> {
    match key_code {
        KeyCode::D => Some(Message::RequestDayChange),
        KeyCode::I => Some(Message::ChangeView(ViewId::BookSingle)),
        KeyCode::O => Some(Message::ChangeView(ViewId::FastDayStart)),
        KeyCode::L => Some(Message::ChangeView(ViewId::FastDayEnd)),
        KeyCode::S => Some(Message::ChangeView(ViewId::BookIssueStart)),
        KeyCode::E => Some(Message::ChangeView(ViewId::BookIssueEnd)),
        KeyCode::X => Some(Message::ChangeView(ViewId::Export)),
        KeyCode::C => Some(Message::CopyValue),
        KeyCode::Key1 => Some(Message::ChangeView(ViewId::CurrentDayUi)),
        KeyCode::Key2 => Some(Message::ChangeView(ViewId::Book)),
        KeyCode::Enter | KeyCode::NumpadEnter => Some(Message::SubmitCurrent(StayActive::Yes)),
        _ => None,
    }
}

fn handle_keyboard_event(key_event: iced_winit::keyboard::Event) -> Option<Message> {
    use iced_winit::keyboard::Event::*;
    match key_event {
        KeyPressed {
            key_code,
            modifiers,
        } => {
            if modifiers.is_empty() {
                match key_code {
                    KeyCode::Escape => Some(Message::Exit),
                    KeyCode::Tab => Some(Message::Next),
                    KeyCode::I => Some(Message::ChangeView(ViewId::BookSingle)),
                    KeyCode::O => Some(Message::ChangeView(ViewId::FastDayStart)),
                    KeyCode::L => Some(Message::ChangeView(ViewId::FastDayEnd)),
                    KeyCode::S => Some(Message::ChangeView(ViewId::BookIssueStart)),
                    KeyCode::E => Some(Message::ChangeView(ViewId::BookIssueEnd)),
                    KeyCode::X => Some(Message::ChangeView(ViewId::Export)),
                    KeyCode::Key1 => Some(Message::ChangeView(ViewId::CurrentDayUi)),
                    KeyCode::Key2 => Some(Message::ChangeView(ViewId::Book)),
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        Some(Message::SubmitCurrent(StayActive::Default))
                    }
                    _ => None,
                }
            } else if modifiers.shift() {
                match key_code {
                    KeyCode::Tab => Some(Message::Previous),
                    _ => None,
                }
            } else if modifiers.control() {
                match key_code {
                    KeyCode::Enter | KeyCode::NumpadEnter => {
                        Some(Message::SubmitCurrent(StayActive::Yes))
                    }
                    key_code => handle_control_shortcuts(key_code),
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

enum CurrentView {
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
    fn view_id(&self) -> ViewId {
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

    fn create(id: ViewId, settings: SettingsRef, active_day: Option<&ActiveDay>) -> CurrentView {
        let guard = settings.load();

        match id {
            ViewId::Book => CurrentView::Book(Book::new(&guard)),
            ViewId::FastDayStart => {
                CurrentView::Fds(FastDayStart::for_work_day(settings, active_day))
            }
            ViewId::FastDayEnd => CurrentView::Fde(FastDayEnd::for_work_day(settings, active_day)),
            ViewId::BookSingle => {
                CurrentView::Bs(BookSingleUI::for_active_day(settings, active_day))
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

    fn create_for_edit(
        value: Action,
        settings: SettingsRef,
        active_day: Option<&ActiveDay>,
    ) -> CurrentView {
        match value {
            Action::Work(a) => {
                let mut ui = BookSingleUI::for_active_day(settings, active_day);
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
            _ => CurrentView::create(ViewId::CurrentDayUi, settings, active_day),
        }
    }
}

trait MainView {
    fn view(&mut self, settings: &Settings) -> QElement;

    fn update(&mut self, msg: Message) -> Option<Message>;
}

type QElement<'a> = Element<'a, Message, <Quarble as iced_winit::Program>::Renderer>;

struct Exit;

impl MainView for Exit {
    fn view(&mut self, _settings: &Settings) -> QElement {
        Text::new("exiting ...").into()
    }

    fn update(&mut self, _msg: Message) -> Option<Message> {
        None
    }
}

fn input_message(s: &str, actions: &BTreeSet<Action>) -> String {
    match min_max_booked(actions) {
        (None, None) => s.to_string(),
        (Some(start), None) | (None, Some(start)) => format!("{}: First action on {}", s, start),
        (Some(start), Some(end)) => format!("{}: Already booked from {} to {}", s, start, end),
    }
}

fn day_info_message(d: Option<&ActiveDay>) -> String {
    if let Some(d) = d {
        match min_max_booked(d.actions()) {
            (None, None) => format!("{} - nothing booked", d.get_day()),
            (Some(start), None) | (None, Some(start)) => {
                format!("{}: first action on {}", d.get_day(), start)
            }
            (Some(start), Some(end)) => {
                format!("{}: booked from {} to {}", d.get_day(), start, end)
            }
        }
    } else {
        "No day selected".to_string()
    }
}

fn unbooked_time(d: Option<&ActiveDay>) -> Vec<TimeLimit> {
    d.map(|d| unbooked_time_for_day(d.actions()))
        .unwrap_or_default()
}

fn min_max_booked(actions: &BTreeSet<Action>) -> (Option<Time>, Option<Time>) {
    let mut iter = actions.iter();
    let first = iter.next();
    let last = iter.next_back();
    match (first, last) {
        (None, _) => (None, None),
        (Some(first), None) => {
            let (s, e) = first.times();
            (Some(s), e)
        }
        (Some(first), Some(last)) => {
            let (s, _) = first.times();
            let (e1, e2) = last.times();
            if e2.is_some() {
                (Some(s), e2)
            } else {
                (Some(s), Some(e1))
            }
        }
    }
}

fn unbooked_time_for_day(actions: &BTreeSet<Action>) -> Vec<TimeLimit> {
    let mut result = Vec::new();
    let mut current_limit = TimeLimit::default();
    for action in actions {
        let (min, max) = action.times();
        let (f, s) = if let Some(max) = max {
            let sep = TimeLimit::simple(min, max);
            current_limit.split(sep)
        } else {
            current_limit.split_at(min)
        };
        match (f, s) {
            (TimeLimit::EMPTY, TimeLimit::EMPTY) => (),
            (TimeLimit::EMPTY, s) => current_limit = s,
            (f, TimeLimit::EMPTY) => current_limit = f,
            (f, s) => {
                result.push(f);
                current_limit = s;
            }
        }
    }

    result.push(current_limit);

    result
}

fn text<'a>(t: impl Into<String>) -> QElement<'a> {
    Text::new(t).into()
}

fn time_info<'a>(now: Time, v: ParseResult<Time, ()>) -> QElement<'a> {
    Text::new(
        v.get_with_default(now)
            .map(|e| e.to_string())
            .unwrap_or_else(|| "invalid".to_string()),
    )
    .into()
}
