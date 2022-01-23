use std::collections::BTreeSet;
use std::rc::Rc;

use arc_swap::ArcSwap;
use iced_core::{Color, Padding};
use iced_native::clipboard;
use iced_wgpu::Text;
use iced_winit::settings::SettingsWindowConfigurator;
use iced_winit::widget::{Column, Container};
use iced_winit::Program;
use iced_winit::{Command, Subscription};
use iced_winit::{Element, Mode};

use current_view::CurrentView;
pub use message::Message;
use stay_active::StayActive;
pub use view_id::ViewId;

use crate::conf::{update_settings, SettingsRef};
use crate::data::{
    Action, ActiveDay, Day, RecentIssues, RecentIssuesData, RecentIssuesRef, TimedAction,
};
use crate::db::DB;
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::time_limit::{TimeRange};
use crate::ui::current_day::CurrentDayMessage;
use crate::ui::export::DayExportMessage;
use crate::ui::main_action::MainAction;
use crate::ui::message::{DeleteAction, EditAction};
use crate::ui::recent_issues_view::RecentIssuesView;
use crate::ui::tab_bar::TabBar;
use crate::ui::util::v_space;
use crate::ui::window_configurator::{DisplaySelection, MyWindowConfigurator};
use crate::Settings;

mod book;
mod book_single;
mod clip_read;
mod current_day;
mod current_view;
mod entry_edit;
mod export;
pub mod fast_day_end;
pub mod fast_day_start;
mod issue_end_edit;
mod issue_start_edit;
mod keyboard_handler;
pub mod main_action;
mod message;
mod recent_issues_view;
mod stay_active;
mod style;
mod tab_bar;
mod top_bar;
mod util;
mod view_id;
mod window_configurator;
mod work_entry_edit;
mod work_start_edit;

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
        default_text_size: 18,
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
    initial_view: ViewId,
    tab_bar: TabBar,
    recent_issues: RecentIssuesRef,
    recent_view: RecentIssuesView,
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
                            self.recent_issues.clone(),
                            self.active_day.as_ref(),
                        );
                        message = Some(Message::Cd(CurrentDayMessage::StartDayChange));
                    }
                }
                Message::ChangeDayRelative(amount, forwarder) => {
                    if let Some(active) = &self.active_day {
                        let day = active
                            .get_day()
                            .add_with_forwarder(amount, forwarder.as_ref());
                        message = Some(Message::ChangeDay(day))
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
                        self.recent_view.refresh();
                        self.current_view = CurrentView::create(
                            view_id,
                            self.settings.clone(),
                            self.recent_issues.clone(),
                            self.active_day.as_ref(),
                        );
                    }
                }
                Message::RefreshView => {
                    self.tab_bar.set_active_view(self.current_view.view_id());
                    self.recent_view.refresh();
                    self.current_view = CurrentView::create(
                        self.current_view.view_id(),
                        self.settings.clone(),
                        self.recent_issues.clone(),
                        self.active_day.as_ref(),
                    );
                }
                Message::Reset => {
                    message = Some(Message::ChangeView(self.initial_view));
                }
                Message::EditAction(EditAction(action)) => {
                    self.recent_view.refresh();
                    self.current_view = CurrentView::create_for_edit(
                        *action,
                        self.settings.clone(),
                        self.recent_issues.clone(),
                        self.active_day.as_ref(),
                    );
                    self.tab_bar.set_active_view(self.current_view.view_id());
                }
                Message::DeleteAction(DeleteAction(_stay_active, action)) => {
                    if let Some(ref mut active_day) = self.active_day {
                        if active_day.actions_mut().remove(&action) {
                            message = match self.db.store_day(active_day) {
                                Ok(()) => Some(Message::RefreshView),
                                Err(e) => Some(Message::Error(format!("{:?}", e))),
                            }
                        } else {
                            message =
                                Some(Message::Error("Cannot find action to delete".to_string()));
                        }
                    }
                }
                Message::StoreAction(stay_active, action) => {
                    if let Some(ref mut active_day) = self.active_day {
                        if let Some(issue) = action.issue() {
                            self.recent_issues
                                .issue_used_with_comment(issue, action.description())
                        }
                        active_day.add_action(action);
                        message = store_active_day(
                            &self.db,
                            &self.settings.load(),
                            stay_active,
                            active_day,
                            self.recent_view.export_data(),
                        );
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
                            if let Some(issue) = update.issue() {
                                self.recent_issues
                                    .issue_used_with_comment(issue, update.description());
                            }
                            actions.insert(*update);

                            message = store_active_day(
                                &self.db,
                                &self.settings.load(),
                                stay_active,
                                active_day,
                                self.recent_view.export_data(),
                            );
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
                            self.recent_issues.clone(),
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
        let view_id = self.current_view.view_id();
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
        let mut main = Column::with_children(vec![self.tab_bar.view(), element]);
        if view_id.show_recent() {
            main = main
                .push(v_space(style::SPACE))
                .push(self.recent_view.view(&settings));
        }

        let main: QElement = main.into();
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

        let recent = db.load_recent().unwrap_or_default();
        let recent_issues = RecentIssues::new(recent, settings.clone());
        let recent_issues = RecentIssuesRef::new(recent_issues);

        let current_view = CurrentView::create(
            flags.initial_view,
            settings.clone(),
            recent_issues.clone(),
            active_day.as_ref(),
        );

        let recent_view = RecentIssuesView::create(recent_issues.clone());

        let mut quarble = Quarble {
            current_view,
            settings,
            db,
            active_day,
            initial_view: flags.initial_view,
            tab_bar: TabBar::new(flags.initial_view),
            recent_view,
            recent_issues,
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
        iced_winit::subscription::events_with(keyboard_handler::global_keyboard_handler)
    }

    fn should_exit(&self) -> bool {
        matches!(&self.current_view, CurrentView::Exit(_))
    }
}

fn store_active_day(
    db: &DB,
    settings: &Settings,
    stay_active: StayActive,
    active_day: &ActiveDay,
    recent_data: RecentIssuesData,
) -> Option<Message> {
    let issue_store_msg = match db.store_day(active_day) {
        Ok(()) => Some(Message::StoreSuccess(stay_active.apply_settings(settings))),
        Err(e) => Some(Message::Error(format!("{:?}", e))),
    };

    if let Err(e) = db.store_recent(&recent_data) {
        eprintln!("Storing recent issues failed: {:?}", e);
    }

    issue_store_msg
}

trait MainView {
    fn view(&mut self, settings: &Settings) -> QElement;

    fn update(&mut self, msg: Message) -> Option<Message>;
}

type QElement<'a> = Element<'a, Message, <Quarble as iced_winit::Program>::Renderer>;

pub struct Exit;

impl MainView for Exit {
    fn view(&mut self, _settings: &Settings) -> QElement {
        Text::new("exiting ...").into()
    }

    fn update(&mut self, _msg: Message) -> Option<Message> {
        None
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

fn unbooked_time(d: Option<&ActiveDay>) -> Vec<TimeRange> {
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

fn unbooked_time_for_day(actions: &BTreeSet<Action>) -> Vec<TimeRange> {
    let mut result = Vec::new();
    let mut current_limit = TimeRange::default();
    for action in actions {
        let (min, max) = action.times();
        let (f, s) = if let Some(max) = max {
            let sep = TimeRange::new(min, max);
            current_limit.split(sep)
        } else {
            current_limit.split_at(min)
        };

        if !f.is_empty() && !s.is_empty() {
            result.push(f);
            current_limit = s;
        } else if !f.is_empty() {
            current_limit = f;
        } else if !s.is_empty() {
            current_limit = s;
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
