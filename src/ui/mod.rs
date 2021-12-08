use std::rc::Rc;
use std::sync::{mpsc, Arc};
use std::thread::current;

use anyhow::Result;
use arc_swap::ArcSwap;
use iced_wgpu::{Container, Row, Scrollable, Text};
use iced_winit::renderer::Quad;
use iced_winit::widget::{scrollable, Column, Rule, Space};
use iced_winit::winit::event_loop::EventLoopWindowTarget;
use iced_winit::winit::platform::unix::{EventLoopWindowTargetExtUnix, WindowBuilderExtUnix};
use iced_winit::winit::window::WindowBuilder;
use iced_winit::Event;
use iced_winit::{event, Command, Subscription};
use iced_winit::{Element, Mode};

use crate::conf::{InitialAction, MainAction, Settings};
use crate::data::WorkStart;
use entry_edit::EntryEdit;
use crate::ui::book::Book;
use crate::ui::window_configurator::DisplaySelection;
use crate::ui::Message::{UpdateDescription, UpdateEnd, UpdateStart};
use crate::ui::work_entry_edit::WorkEntryEdit;
use crate::ui::work_start_edit::WorkStartEdit;

pub mod main_action;
mod work_entry_edit;
mod style;
mod window_configurator;
mod entry_edit;
mod work_start_edit;
mod util;
mod book;
mod time;


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
}
impl Default for Message {
    fn default() -> Self {
        Message::Update
    }
}

pub fn show_ui(main_action: MainAction) -> Rc<ArcSwap<Settings>> {
    let config_settings = main_action.settings.clone();
    let settings = iced_winit::Settings {
        id: Some("tmenu".to_string()),
        window: Default::default(),
        flags: main_action,
        exit_on_close_request: true,
        window_configurator: Some(Arc::new(window_configurator::MyWindowConfigurator {
            display_selection: DisplaySelection::Largest,
        })),
    };

    let renderer_settings = iced_wgpu::Settings {
        antialiasing: Some(iced_wgpu::settings::Antialiasing::MSAAx4),
        ..iced_wgpu::Settings::from_env()
    };
    iced_winit::application::run::<
        Quarble,
        iced_futures::executor::ThreadPool,
        iced_wgpu::window::Compositor,
    >(settings, renderer_settings)
    .unwrap();

    config_settings
}

pub struct Quarble {
    current_view: CurrentView,
    settings: Rc<ArcSwap<Settings>>,
}

impl iced_winit::Program for Quarble {
    type Renderer = iced_wgpu::Renderer;
    type Message = Message;

    fn update(&mut self, mut message: Message) -> Command<Message> {
        loop {
            match message {
                Message::Update => break,
                Message::Exit => {
                    self.current_view = CurrentView::Exit(Exit);
                    break;
                }
                Message::Book => {
                    if let CurrentView::Book(_) = &self.current_view {
                    } else {
                        self.current_view = CurrentView::Book(Book::new());
                    }
                    break;
                }
                Message::View => {
                    if let CurrentView::Show(_) = &self.current_view {
                    } else {
                        self.current_view = CurrentView::Show(ViewBookings::new());
                    }
                    break;
                }
                m => match &mut self.current_view {
                    CurrentView::Book(b) => {
                        eprintln!("Sending {:?} to book", &m);
                        if let Some(f) = b.update(m) {
                            message = f;
                        } else {
                            break;
                        }
                    }
                    _ => break,
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
            CurrentView::Exit(exit) => exit.view(&settings),
        }
    }
}

impl iced_winit::Application for Quarble {
    type Flags = MainAction;

    fn new(flags: MainAction) -> (Self, Command<Message>) {
        let current_view = match flags.initial_action {
            InitialAction::Book => CurrentView::Book(Book::new()),
            InitialAction::Show => CurrentView::Show(Box::new(ViewBookings {})),
        };

        let settings = flags.settings;

        (
            Quarble {
                current_view,
                settings,
            },
            Command::none(),
        )
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
    use iced_winit::keyboard::Modifiers;
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
    use iced_winit::keyboard::Modifiers;
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
