use std::rc::Rc;

use arc_swap::ArcSwap;
use iced_wgpu::Text;
use iced_winit::settings::SettingsWindowConfigurator;
use iced_winit::{event, Command, Subscription};
use iced_winit::{Element, Mode};
use iced_winit::{Event, Program};

use crate::conf::{InitialAction, MainAction, Settings};
use crate::data::{Action, Day, WorkDay};
use crate::db::DB;
use crate::ui::book::Book;
use crate::ui::fast_day_start::{FastDayStart, FastDayStartMessage};
use crate::ui::window_configurator::{DisplaySelection, MyWindowConfigurator};

mod book;
mod entry_edit;
mod fast_day_start;
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
    FDS(FastDayStartMessage),
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
    active_day: Option<WorkDay>,
    active_day_dirty: bool,
}

impl iced_winit::Program for Quarble {
    type Renderer = iced_wgpu::Renderer;
    type Message = Message;

    fn update(&mut self, mut message: Message) -> Command<Message> {
        let mut message = Some(message);
        while let Some(current) = message.take() {
            match current {
                Message::Update => {}
                Message::Error(msg) => eprintln!("Got an error: {}", msg),
                Message::Exit => {
                    self.current_view = CurrentView::Exit(Exit);
                }
                Message::Book => {
                    if let CurrentView::Book(_) = &self.current_view {
                    } else {
                        self.current_view = CurrentView::Book(Book::new());
                    }
                }
                Message::View => {
                    if let CurrentView::Show(_) = &self.current_view {
                    } else {
                        self.current_view = CurrentView::Show(ViewBookings::new());
                    }
                }
                Message::StoreAction(action) => {
                    if let Some(ref mut active_day) = self.active_day {
                        active_day.add_action(action);
                        message = match self.db.store_day(active_day.get_day(), active_day) {
                            Ok(()) => Some(Message::StoreSuccess),
                            Err(e) => Some(Message::Error(format!("{:?}", e))),
                        };
                    }
                }
                m => match &mut self.current_view {
                    CurrentView::Book(b) => {
                        eprintln!("Sending {:?} to book", &m);
                        message = b.update(m);
                    }
                    CurrentView::FDS(fds) => {
                        message = fds.update(m);
                    }
                    _ => {}
                },
            }
        }
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message, Self::Renderer> {
        let settings = self.settings.load();
        match &mut self.current_view {
            CurrentView::Book(book) => book.view(&settings),
            CurrentView::Show(show) => show.view(&settings),
            CurrentView::FDS(fds) => fds.view(&settings),
            CurrentView::Exit(exit) => exit.view(&settings),
        }
    }
}

impl iced_winit::Application for Quarble {
    type Flags = MainAction;

    fn new(flags: MainAction) -> (Self, Command<Message>) {
        let db = flags.db;

        let (current_view, active_day) = match flags.initial_action {
            InitialAction::Book => (CurrentView::Book(Book::new()), Ok(None)),
            InitialAction::Show => (
                CurrentView::Show(Box::new(ViewBookings {})),
                db.get_day(Day::today()).map(Option::from),
            ),
            InitialAction::FastStartDay => (
                CurrentView::FDS(FastDayStart::new()),
                db.get_day(Day::today()).map(Option::from),
            ),
        };

        let settings = flags.settings;
        let (initial_message, active_day) = match active_day {
            Ok(active_day) => (None, active_day),
            Err(e) => (Some(Message::Error(format!("{:?}", e))), None),
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

    fn should_exit(&self) -> bool {
        if let CurrentView::Exit(_) = &self.current_view {
            true
        } else {
            false
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_winit::subscription::events_with(global_keyboard_handler)
    }
}

fn global_keyboard_handler(event: Event, status: iced_winit::event::Status) -> Option<Message> {
    if let event::Status::Captured = status {
        if let Event::Keyboard(kb) = event {
            handle_control_keyboard_event(kb)
        } else {
            None
        }
    } else {
        if let Event::Keyboard(kb) = event {
            handle_keyboard_event(kb)
        } else {
            None
        }
    }
}

fn handle_control_keyboard_event(key_event: iced_winit::keyboard::Event) -> Option<Message> {
    use iced_core::keyboard::KeyCode;
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
            } else {
                None
            }
        }
        _ => None,
    }
}

fn handle_keyboard_event(key_event: iced_winit::keyboard::Event) -> Option<Message> {
    use iced_core::keyboard::KeyCode;
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
                    KeyCode::V => Some(Message::View),
                    KeyCode::B => Some(Message::Book),
                    _ => None,
                }
            } else if modifiers.shift() {
                match key_code {
                    KeyCode::Tab => Some(Message::Previous),
                    _ => None,
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
    Show(Box<ViewBookings>),
    FDS(Box<FastDayStart>),
    Exit(Exit),
}

trait MainView {
    fn new() -> Box<Self>;

    fn view<'a>(&'a mut self, settings: &Settings) -> QElement<'a>;

    fn update(&mut self, msg: Message) -> Option<Message>;
}

type QElement<'a> = Element<'a, Message, <Quarble as iced_winit::Program>::Renderer>;

struct ViewBookings {}

impl MainView for ViewBookings {
    fn new() -> Box<Self> {
        Box::new(ViewBookings {})
    }
    fn view<'a>(&'a mut self, _settings: &Settings) -> QElement<'a> {
        Text::new("show").into()
    }
    fn update(&mut self, _msg: Message) -> Option<Message> {
        None
    }
}

struct Exit;

impl MainView for Exit {
    fn new() -> Box<Self> {
        Box::new(Exit)
    }

    fn view<'a>(&'a mut self, _settings: &Settings) -> QElement<'a> {
        Text::new("exiting ...").into()
    }

    fn update(&mut self, _msg: Message) -> Option<Message> {
        None
    }
}
