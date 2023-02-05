use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;

use arc_swap::ArcSwap;

use crate::data::{Day, JiraIssue};
use crate::parsing::time::Time;
use crate::parsing::JiraIssueParser;
use crate::util::{update_arcswap, DefaultTimeline, Timeline, TimelineProvider};

/// Current application state. Shared across all views and widgets
///
/// Typically behind an ArcSwap and therefore immutable
///
/// Use [update_settings] for changing settings
#[derive(Clone, Debug)]
pub struct Settings {
    pub settings_location: Option<PathBuf>,
    pub db_dir: PathBuf,
    pub resolution: chrono::Duration,
    pub write_settings: bool,
    pub active_date: Day,
    pub timeline: Timeline,
    pub issue_parser: JiraIssueParser,
    pub breaks: BreaksConfig,
    pub debug: bool,
    pub close_on_safe: bool,
    pub max_recent_issues: usize,
    pub add_location: bool,
    pub combine_bookings: bool,
}

impl Settings {
    pub fn from_ser(ser: Option<SettingsSer>) -> Self {
        if let Some(s) = ser {
            Self::default().apply_ser(s)
        } else {
            Self::default()
        }
    }

    pub fn apply_ser(&self, ser: SettingsSer) -> Self {
        Self {
            settings_location: self.settings_location.clone(),
            db_dir: ser.db_dir,
            resolution: chrono::Duration::minutes(ser.resolution_minutes as i64),
            write_settings: self.write_settings,
            active_date: self.active_date,
            timeline: self.timeline.clone(),
            issue_parser: JiraIssueParser::new(ser.issue_shortcuts),
            breaks: ser.breaks,
            debug: self.debug,
            close_on_safe: self.close_on_safe,
            max_recent_issues: ser.max_recent_issues as usize,
            add_location: ser.export.add_location,
            combine_bookings: ser.export.combine_bookings,
        }
    }

    pub fn into_settings_ref(self) -> SettingsRef {
        into_settings_ref(self)
    }
}

pub type SettingsRef = Rc<ArcSwap<Settings>>;

pub fn into_settings_ref(s: Settings) -> SettingsRef {
    Rc::new(ArcSwap::new(Arc::new(s)))
}

/// Update current [Settings]
///
/// Is not thread-safe and does protect against lost updates
pub fn update_settings(settings: &SettingsRef, f: impl FnOnce(&mut Settings)) {
    update_arcswap(settings, |s| {
        f(s);
        s.write_settings = true;
    })
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
            issue_parser: JiraIssueParser::default(),
            breaks: Default::default(),
            debug: false,
            close_on_safe: true,
            max_recent_issues: 10,
            add_location: false,
            combine_bookings: true,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub struct SettingsSer {
    pub db_dir: PathBuf,
    #[serde(default)]
    pub resolution_minutes: u32,
    #[serde(default)]
    pub issue_shortcuts: BTreeMap<char, JiraIssue>,
    #[serde(default)]
    pub breaks: BreaksConfig,
    #[serde(default = "default_max_recent_issues")]
    pub max_recent_issues: u32,
    #[serde(default)]
    pub export: ExportConfig,
}

fn default_true() -> bool {
    true
}

fn default_max_recent_issues() -> u32 {
    10
}

impl SettingsSer {
    pub fn from_settings(settings: &Settings) -> SettingsSer {
        SettingsSer {
            db_dir: settings.db_dir.clone(),
            resolution_minutes: settings.resolution.num_minutes() as u32,
            issue_shortcuts: settings.issue_parser.shortcuts().clone(),
            breaks: settings.breaks.clone(),
            max_recent_issues: settings.max_recent_issues as u32,
            export: ExportConfig {
                add_location: settings.add_location,
                combine_bookings: settings.combine_bookings,
            },
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct BreaksConfig {
    pub min_breaks_minutes: u32,
    pub min_work_time_minutes: u32,
    pub default_break: (Time, Time),
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct ExportConfig {
    pub add_location: bool,
    pub combine_bookings: bool,
}

impl Default for ExportConfig {
    fn default() -> Self {
        ExportConfig {
            add_location: false,
            combine_bookings: true,
        }
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;
    use std::path::Path;

    use crate::conf::{BreaksConfig, SettingsSer};
    use crate::conf::settings::ExportConfig;
    use crate::data::JiraIssue;
    use crate::parsing::time::Time;

    #[test]
    fn test_serialize_settings() {
        let orig = SettingsSer {
            db_dir: Path::new("db/dir").to_owned(),
            resolution_minutes: 15,
            issue_shortcuts: BTreeMap::from_iter(
                vec![
                    (
                        'a',
                        JiraIssue {
                            ident: "A-8".to_string(),
                            description: Some("Agile meeting".to_string()),
                            default_action: Some("meeting".to_string()),
                        },
                    ),
                    (
                        'b',
                        JiraIssue {
                            ident: "A-5".to_string(),
                            description: Some("Project related meeting".to_string()),
                            default_action: None,
                        },
                    ),
                    (
                        'm',
                        JiraIssue {
                            ident: "A-2".to_string(),
                            description: Some("Management".to_string()),
                            default_action: None,
                        },
                    ),
                ]
                .into_iter(),
            ),
            breaks: BreaksConfig {
                min_breaks_minutes: 45,
                min_work_time_minutes: 360,
                default_break: (Time::hm(11, 30), Time::hm(12, 15)),
            },
            max_recent_issues: 15,
            export: ExportConfig {
                add_location: true,
                combine_bookings: false,
            },
        };

        let pretty = serde_json::to_string_pretty(&orig).unwrap();

        let parsed = serde_json::from_str(&pretty).unwrap();
        assert_eq!(orig, parsed);
    }
}
