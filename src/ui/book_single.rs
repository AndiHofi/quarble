use crate::conf::Settings;
use crate::data::{ActiveDay, JiraIssue, Task, Work};
use crate::parsing::parse_result::ParseResult;
use crate::parsing::time::Time;
use crate::parsing::{parse_input, parse_input_rel};
use crate::ui::util::{h_space, v_space};
use crate::ui::{input_message, style, MainView, Message, QElement};
use crate::util;
use iced_native::widget::Row;
use iced_wgpu::TextInput;
use iced_winit::widget::{text_input, Column, Text};

#[derive(Clone, Debug)]
pub enum BookSingleMessage {
    TextChanged(String),
}

pub struct BookSingleUI {
    input_state: text_input::State,
    input: String,
    data: Option<Work>,
    builder: WorkBuilder,
    input_message: String,
}

impl BookSingleUI {
    fn parse_input(&mut self, text: &str) {
        let (text, msg) = if let Some((text, msg)) = text.split_once('#') {
            (text, msg)
        } else {
            (text, "")
        };

        let now = util::time_now().into();

        let entries: Vec<_> = text.splitn(4, ' ').collect();
        self.builder.start = ParseResult::None;
        self.builder.end = ParseResult::None;
        self.builder.task = ParseResult::None;
        self.builder.msg = if msg.is_empty() {
            Some(msg.to_string())
        } else {
            None
        };

        match entries.as_slice() {
            &[s, e, i, m0, ref m @ ..] => {
                if msg.is_empty() {
                    self.builder.start = parse_input_rel(now, s, true);
                    self.builder.end = parse_input(now, e);
                    self.builder.task = parse_issue(i);
                    self.builder.msg = Some(format!("{} {}", m0, m.join(" ")));
                } else {
                    self.builder.task = ParseResult::Invalid(())
                }
            }
            &[s, e, i] => {
                self.builder.start = parse_input_rel(now, s, true);
                self.builder.end = parse_input(now, e);
                self.builder.task = parse_issue(i);
            }
            &[s, i] => {
                self.builder.start = parse_input_rel(now, s, true);
                self.builder.task = parse_issue(i);
            }
            &[i] => {
                self.builder.task = parse_issue(i);
            }
            &[] => {
                self.builder.start = ParseResult::Incomplete;
            }
        };
    }

    pub fn for_active_day(active_day: Option<&ActiveDay>) -> Box<Self> {
        let actions = active_day.map(|a| a.actions()).unwrap_or_default();
        Box::new(Self {
            input_state: Default::default(),
            input: "".to_string(),
            data: None,
            builder: Default::default(),
            input_message: input_message("Book issue", actions),
        })
    }

    fn follow_up_msg(&mut self) -> Option<Message> {
        if let (ParseResult::None, ParseResult::None) =
            (self.builder.clipboard.as_ref(), self.builder.task.as_ref())
        {
            self.builder.clipboard = ParseResult::Incomplete;
            Some(Message::ReadClipboard)
        } else {
            None
        }
    }
}

impl MainView for BookSingleUI {
    fn new() -> Box<Self> {
        Box::new(Self {
            input_state: text_input::State::focused(),
            builder: Default::default(),
            data: None,
            input: String::new(),
            input_message: input_message("Book issue", &[]),
        })
    }

    fn view(&mut self, settings: &Settings) -> QElement {
        let msg = Text::new(&self.input_message);
        let input = TextInput::new(&mut self.input_state, "", &self.input, |s| {
            Message::BS(BookSingleMessage::TextChanged(s))
        })
        .on_submit(
            self.builder
                .try_build(util::time_now().into(), settings)
                .map(|e| Message::StoreAction(crate::data::Action::Work(e)))
                .unwrap_or_default(),
        );

        let now = util::time_now().into();

        let status = Row::with_children(vec![
            text("Start:"),
            h_space(style::SPACE),
            time_info(now, self.builder.start.clone()),
            h_space(style::DSPACE),
            text("End:"),
            h_space(style::SPACE),
            time_info(now, self.builder.end.clone()),
            h_space(style::DSPACE),
            text("Task:"),
            h_space(style::SPACE),
            task_info(self.builder.task.as_ref()),
            h_space(style::DSPACE),
            text("Message:"),
            h_space(style::SPACE),
            text(
                self.builder
                    .msg
                    .as_ref()
                    .map(|e| e.as_ref())
                    .unwrap_or("<no message>"),
            ),
        ]);

        Column::with_children(vec![
            msg.into(),
            v_space(style::SPACE),
            input.into(),
            v_space(style::SPACE),
            status.into(),
        ])
        .into()
    }

    fn update(&mut self, msg: Message) -> Option<Message> {
        match msg {
            Message::BS(BookSingleMessage::TextChanged(msg)) => {
                self.parse_input(&msg);
                self.input = msg;
                self.follow_up_msg()
            }
            Message::ClipboardValue(v) => match v {
                Some(v) => {
                    match parse_issue_clipboard(&v) {
                        Some(i) => self.builder.task = ParseResult::Valid(Task::Jira(i)),
                        None => self.builder.task = ParseResult::Invalid(()),
                    };
                    eprintln!("{} --> {:?}", v, self.builder.task);
                    self.builder.clipboard = ParseResult::Valid(v);
                    None
                }
                None => {
                    self.builder.clipboard = ParseResult::Invalid(());
                    None
                }
            },
            Message::StoreSuccess => Some(Message::Exit),
            _ => None,
        }
    }
}

fn text<'a>(t: &str) -> QElement<'a> {
    Text::new(t.to_string()).into()
}

fn time_info<'a>(now: Time, v: ParseResult<Time, ()>) -> QElement<'a> {
    Text::new(
        v.get_with_default(now)
            .map(|e| e.to_string())
            .unwrap_or("invalid".to_string()),
    )
    .into()
}

fn task_info<'a>(v: ParseResult<&Task, &()>) -> QElement<'a> {
    match v {
        ParseResult::Valid(t) => match t {
            Task::Jira(i) => text(&i.ident),
            Task::Admin => text("Admin"),
            Task::Meeting => text("Meeting"),
        },
        ParseResult::Invalid(_) => text("invalid"),
        ParseResult::Incomplete => text("incomplete"),
        ParseResult::None => text("<clipboard>"),
    }
}

fn parse_issue_clipboard(input: &str) -> Option<JiraIssue> {
    let input = input.trim();
    if let Some((key, input)) = input.split_once('-') {
        if !key.chars().all(|c| c.is_ascii_alphabetic()) {
            return None;
        }
        if let Some((num, rest)) = input.split_once(&[' ', '\t', ':'][..]) {
            if !num.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }

            let description: String = rest.chars().skip_while(|ch| ch.is_alphanumeric()).collect();

            Some(JiraIssue {
                ident: format!("{}-{}", key, num),
                description: if description.is_empty() {
                    None
                } else {
                    Some(description)
                },
                default_action: None,
            })
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_issue(input: &str) -> ParseResult<Task, ()> {
    if input == "m" {
        ParseResult::Valid(Task::Meeting)
    } else if input == "a" {
        ParseResult::Valid(Task::Admin)
    } else if input == "c" {
        ParseResult::None
    } else {
        if let Some((key, num)) = input.split_once('-') {
            if key.chars().all(|ch| ch.is_ascii_alphabetic())
                && num.chars().all(|ch| ch.is_ascii_digit())
                && key.len() >= 1
                && num.len() >= 1
            {
                ParseResult::Valid(Task::Jira(JiraIssue {
                    ident: format!("{}-{}", key, num),
                    description: None,
                    default_action: None,
                }))
            } else {
                ParseResult::Invalid(())
            }
        } else {
            ParseResult::Invalid(())
        }
    }
}

#[derive(Default, Debug)]
struct WorkBuilder {
    start: ParseResult<Time, ()>,
    end: ParseResult<Time, ()>,
    task: ParseResult<Task, ()>,
    msg: Option<String>,
    clipboard: ParseResult<String, ()>,
}

impl WorkBuilder {
    fn needs_clipboard(&self) -> bool {
        if let ParseResult::None = self.task {
            true
        } else {
            false
        }
    }

    fn apply_clipboard(&mut self, value: Option<String>) {
        if let ParseResult::None = self.task {
            if let Some(e) = value {
                if let Some(ji) = parse_issue_clipboard(&e) {
                    self.task = ParseResult::Valid(Task::Jira(ji));
                } else {
                    self.task = ParseResult::Invalid(());
                }
            } else {
                self.task = ParseResult::Incomplete;
            }
        } else {
            eprintln!("Cannot apply clipboard");
        }
    }

    fn try_build(&self, now: Time, _settings: &Settings) -> Option<Work> {
        let start = self.start.get_with_default(now);

        let end = self.end.get_with_default(now);

        let task = match self.task {
            ParseResult::None => self
                .clipboard
                .get_ref()
                .and_then(|e| parse_issue_clipboard(e))
                .map(|i| Task::Jira(i)),
            ParseResult::Valid(ref t) => Some(t.clone()),
            _ => None,
        };

        match (start, end, task) {
            (Some(start), Some(end), Some(task)) => {
                let description = if let Some(ref d) = self.msg {
                    d.to_string()
                } else {
                    match task {
                        Task::Jira(JiraIssue {
                            description: Some(ref description),
                            ..
                        }) => description.to_string(),
                        _ => return None,
                    }
                };

                Some(Work {
                    start: start.into(),
                    end: end.into(),
                    task: task.clone(),
                    description,
                })
            }
            _ => None,
        }
    }
}
