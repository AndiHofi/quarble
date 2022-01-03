use std::collections::BTreeSet;
use std::rc::Rc;

use arc_swap::ArcSwap;
use iced_core::keyboard::{KeyCode, Modifiers};
use iced_core::Padding;
use iced_wgpu::Text;
use iced_winit::settings::SettingsWindowConfigurator;
use iced_winit::widget::Container;
use iced_winit::{event, Command, Subscription};
use iced_winit::{Element, Mode};
use iced_winit::{Event, Program};

use crate::conf::{InitialAction, MainAction, Settings};
use crate::data::{Action, ActiveDay, Day, TimedAction};
use crate::db::DB;
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::time_limit::TimeLimit;
use crate::ui::book::Book;
use crate::ui::book_single::{BookSingleMessage, BookSingleUI};
use crate::ui::current_day::CurrentDayUI;
use crate::ui::fast_day_end::{FastDayEnd, FastDayEndMessage};
use crate::ui::fast_day_start::{FastDayStart, FastDayStartMessage};
use crate::ui::issue_end_edit::{IssueEndEdit, IssueEndMessage};
use crate::ui::issue_start_edit::{IssueStartEdit, IssueStartMessage};
use crate::ui::window_configurator::{DisplaySelection, MyWindowConfigurator};
use iced_native::clipboard;

mod book;
mod book_single;
mod clip_read;
mod current_day;
mod entry_edit;
mod fast_day_end;
mod fast_day_start;
mod issue_end_edit;
mod issue_start_edit;
pub mod main_action;
mod style;
mod util;
mod window_configurator;
mod work_entry_edit;
mod work_start_edit;

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
    Book,
    View,
    CurrentDayUI,
    FastDayStart,
    FastDayEnd,
    BookSingle,
    BookIssueStart,
    BookIssueEnd,
    ReadClipboard,
    ClipboardValue(Option<String>),
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
    Fds(FastDayStartMessage),
    Fde(FastDayEndMessage),
    CdUi(CurrentDayUI),
    Bs(BookSingleMessage),
    Is(IssueStartMessage),
    Ie(IssueEndMessage),
    StoreAction(Action),
    StoreSuccess,
    Error(String),
}

impl Default for Message {
    fn default() -> Self {
        Message::Update
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
    settings: Rc<ArcSwap<Settings>>,
    db: DB,
    active_day: Option<ActiveDay>,
    active_day_dirty: bool,
}

impl iced_winit::Program for Quarble {
    type Renderer = iced_wgpu::Renderer;
    type Message = Message;

    fn update(&mut self, message: Message) -> Command<Message> {
        let mut message = Some(message);
        while let Some(current) = message.take() {
            let settings = self.settings.load();
            match current {
                Message::Error(msg) => eprintln!("Got an error: {}", msg),
                Message::Exit => {
                    self.current_view = CurrentView::Exit(Exit);
                }
                Message::Book => {
                    if let CurrentView::Book(_) = &self.current_view {
                    } else {
                        self.current_view = CurrentView::Book(Book::new(&settings));
                    }
                }
                Message::View => {
                    if let CurrentView::Show(_) = &self.current_view {
                    } else {
                        self.current_view = CurrentView::Show(ViewBookings::new(&settings));
                    }
                }
                Message::CurrentDayUI => match &self.current_view {
                    CurrentView::CdUi(_) => (),
                    _ => {
                        self.current_view = CurrentView::CdUi(CurrentDayUI::for_active_day(
                            self.active_day.as_ref(),
                        ))
                    }
                },
                Message::FastDayStart => {
                    if let CurrentView::Fds(_) = &self.current_view {
                    } else {
                        self.current_view = CurrentView::Fds(FastDayStart::for_work_day(
                            &settings,
                            self.active_day.as_ref(),
                        ));
                    }
                }
                Message::FastDayEnd => match &self.current_view {
                    CurrentView::Fde(_) => (),
                    _ => {
                        self.current_view = CurrentView::Fde(FastDayEnd::for_work_day(
                            &settings,
                            self.active_day.as_ref(),
                        ));
                    }
                },
                Message::BookSingle => match &self.current_view {
                    CurrentView::Bs(_) => (),
                    _ => {
                        let settings = self.settings.load_full();
                        self.current_view = CurrentView::Bs(BookSingleUI::for_active_day(
                            settings,
                            self.active_day.as_ref(),
                        ));
                    }
                },
                Message::BookIssueStart => match &self.current_view {
                    CurrentView::Is(_) => (),
                    _ => {
                        let settings = self.settings.load_full();
                        self.current_view = CurrentView::Is(IssueStartEdit::for_active_day(
                            settings,
                            self.active_day.as_ref(),
                        ));
                    }
                },
                Message::BookIssueEnd => match &self.current_view {
                    CurrentView::Ie(_) => (),
                    _ => {
                        let settings = self.settings.load_full();
                        self.current_view = CurrentView::Ie(IssueEndEdit::for_active_day(
                            settings,
                            self.active_day.as_ref(),
                        ))
                    }
                },
                Message::StoreAction(action) => {
                    if let Some(ref mut active_day) = self.active_day {
                        active_day.add_action(action);
                        message = match self.db.store_day(active_day.get_day(), active_day) {
                            Ok(()) => Some(Message::StoreSuccess),
                            Err(e) => Some(Message::Error(format!("{:?}", e))),
                        };
                    }
                }
                Message::ReadClipboard => {
                    eprintln!("Reading clipboard");
                    let clipboard = iced_native::command::Action::Clipboard(
                        clipboard::Action::Read(Box::new(move |v| {
                            eprintln!("got clipboard: '{:?}'", v);
                            Message::ClipboardValue(v)
                        })),
                    );
                    return Command::single(clipboard);
                }
                m => match &mut self.current_view {
                    CurrentView::Book(b) => {
                        eprintln!("Sending {:?} to book", &m);
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
            CurrentView::Show(show) => show.view(&settings),
            CurrentView::Fds(fds) => fds.view(&settings),
            CurrentView::Fde(fde) => fde.view(&settings),
            CurrentView::CdUi(cdui) => cdui.view(&settings),
            CurrentView::Bs(bs) => bs.view(&settings),
            CurrentView::Exit(exit) => exit.view(&settings),
            CurrentView::Is(is) => is.view(&settings),
            CurrentView::Ie(ie) => ie.view(&settings),
        };
        Container::new(content)
            .padding(Padding::new(style::WINDOW_PADDING))
            .into()
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

        let guard = settings.load();

        let current_view = match flags.initial_action {
            InitialAction::Book => CurrentView::Book(Book::new(&guard)),
            InitialAction::Show => CurrentView::Show(Box::new(ViewBookings {})),
            InitialAction::FastStartDay => {
                CurrentView::Fds(FastDayStart::for_work_day(&guard, active_day.as_ref()))
            }
            InitialAction::FastEndDay => {
                CurrentView::Fde(FastDayEnd::for_work_day(&guard, active_day.as_ref()))
            }
            InitialAction::BookSingle => CurrentView::Bs(BookSingleUI::for_active_day(
                settings.load_full(),
                active_day.as_ref(),
            )),
            InitialAction::IssueStart => CurrentView::Is(IssueStartEdit::for_active_day(
                settings.load_full(),
                active_day.as_ref(),
            )),
            InitialAction::IssueEnd => CurrentView::Ie(IssueEndEdit::for_active_day(
                settings.load_full(),
                active_day.as_ref(),
            )),
        };

        let mut quarble = Quarble {
            current_view,
            settings,
            db,
            active_day,
            active_day_dirty: false,
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
        KeyCode::I => Some(Message::BookSingle),
        KeyCode::O => Some(Message::FastDayStart),
        KeyCode::L => Some(Message::FastDayEnd),
        KeyCode::S => Some(Message::BookIssueStart),
        KeyCode::E => Some(Message::BookIssueEnd),
        KeyCode::Key1 => Some(Message::CurrentDayUI),
        KeyCode::Key2 => Some(Message::Book),
        KeyCode::Key3 => Some(Message::View),
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
                    KeyCode::I => Some(Message::BookSingle),
                    KeyCode::O => Some(Message::FastDayStart),
                    KeyCode::L => Some(Message::FastDayEnd),
                    KeyCode::S => Some(Message::BookIssueStart),
                    KeyCode::E => Some(Message::BookIssueEnd),
                    KeyCode::Key1 => Some(Message::CurrentDayUI),
                    KeyCode::Key2 => Some(Message::Book),
                    KeyCode::Key3 => Some(Message::View),
                    _ => None,
                }
            } else if modifiers.shift() {
                match key_code {
                    KeyCode::Tab => Some(Message::Previous),
                    _ => None,
                }
            } else if modifiers.control() {
                handle_control_shortcuts(key_code)
            } else {
                None
            }
        }
        _ => None,
    }
}

enum CurrentView {
    Book(Box<Book>),
    Show(Box<ViewBookings>),
    Fds(Box<FastDayStart>),
    Fde(Box<FastDayEnd>),
    CdUi(Box<CurrentDayUI>),
    Bs(Box<BookSingleUI>),
    Is(Box<IssueStartEdit>),
    Ie(Box<IssueEndEdit>),
    Exit(Exit),
}

trait MainView {
    fn view(&mut self, settings: &Settings) -> QElement;

    fn update(&mut self, msg: Message) -> Option<Message>;
}

type QElement<'a> = Element<'a, Message, <Quarble as iced_winit::Program>::Renderer>;

struct ViewBookings {}

impl ViewBookings {
    pub fn new(_settings: &Settings) -> Box<Self> {
        Box::new(ViewBookings {})
    }
}

impl MainView for ViewBookings {
    fn view(&mut self, _settings: &Settings) -> QElement {
        Text::new("show").into()
    }
    fn update(&mut self, _msg: Message) -> Option<Message> {
        None
    }
}

struct Exit;

impl Exit {
    pub fn new(_settings: &Settings) -> Box<Self> {
        Box::new(Exit)
    }
}

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
