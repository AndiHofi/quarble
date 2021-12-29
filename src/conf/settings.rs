use crate::data::Day;
use crate::util::{DefaultTimeline, Timeline, TimelineProvider};
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Debug)]
pub struct Settings {
    pub settings_location: Option<PathBuf>,
    pub db_dir: PathBuf,
    pub resolution: chrono::Duration,
    pub write_settings: bool,
    pub active_date: Day,
    pub timeline: Timeline,
}

impl Settings {
    pub fn with_timeline<T: TimelineProvider + 'static>(mut self, timeline: T) -> Self {
        self.timeline = Arc::new(timeline);
        self
    }
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            settings_location: None,
            db_dir: Default::default(),
            resolution: chrono::Duration::minutes(15),
            write_settings: false,
            active_date: Day::today(),
            timeline: Arc::new(DefaultTimeline),
        }
    }
}

pub struct Story {
    ident: String,
    description: Option<String>,
    default_action: Option<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct SettingsSer {
    pub db_dir: PathBuf,
    pub resolution_minutes: u32,
}

impl SettingsSer {
    pub fn from_settings(settings: &Settings) -> SettingsSer {
        SettingsSer {
            db_dir: settings.db_dir.clone(),
            resolution_minutes: settings.resolution.num_minutes() as u32,
        }
    }
}
