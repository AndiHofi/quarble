use std::path::PathBuf;

#[derive(Debug)]
pub struct Settings {
    pub settings_location: Option<PathBuf>,
    pub db_dir: PathBuf,
    pub resolution: chrono::Duration,
    pub write_settings: bool,
    pub active_date: chrono::NaiveDate,
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
