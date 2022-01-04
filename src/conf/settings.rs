use crate::data::{Day, JiraIssue};
use crate::parsing::IssueParser;
use crate::util::{DefaultTimeline, Timeline, TimelineProvider};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct Settings {
    pub settings_location: Option<PathBuf>,
    pub db_dir: PathBuf,
    pub resolution: chrono::Duration,
    pub write_settings: bool,
    pub active_date: Day,
    pub timeline: Timeline,
    pub issue_parser: IssueParser,
    pub debug: bool,
}

impl Settings {
    pub fn with_timeline<T: TimelineProvider + 'static>(mut self, timeline: T) -> Self {
        self.timeline = Arc::new(timeline);
        self
    }

    pub fn from_ser(ser: Option<SettingsSer>) -> Self {
        if let Some(s) = ser {
            Self {
                db_dir: s.db_dir.clone(),
                resolution: chrono::Duration::minutes(s.resolution_minutes as i64),
                issue_parser: IssueParser::new(s.issue_shortcuts),
                ..Self::default()
            }
        } else {
            Self::default()
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        let timeline = Arc::new(DefaultTimeline);
        Settings {
            settings_location: None,
            db_dir: Default::default(),
            resolution: chrono::Duration::minutes(15),
            write_settings: false,
            active_date: timeline.today(),
            timeline,
            issue_parser: IssueParser::default(),
            debug: false,
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SettingsSer {
    pub db_dir: PathBuf,
    pub resolution_minutes: u32,
    pub issue_shortcuts: BTreeMap<char, JiraIssue>,
}

impl SettingsSer {
    pub fn from_settings(settings: &Settings) -> SettingsSer {
        SettingsSer {
            db_dir: settings.db_dir.clone(),
            resolution_minutes: settings.resolution.num_minutes() as u32,
            issue_shortcuts: settings.issue_parser.shortcuts().clone(),
        }
    }
}
