use crate::data::{Day, JiraIssue};
use crate::parsing::time::Time;
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
    pub breaks: BreaksConfig,
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
                breaks: s.breaks,
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
            breaks: Default::default(),
            debug: false,
        }
    }
}

#[derive(Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct SettingsSer {
    pub db_dir: PathBuf,
    #[serde(default)]
    pub resolution_minutes: u32,
    #[serde(default)]
    pub issue_shortcuts: BTreeMap<char, JiraIssue>,
    #[serde(default)]
    pub breaks: BreaksConfig,
}

impl SettingsSer {
    pub fn from_settings(settings: &Settings) -> SettingsSer {
        SettingsSer {
            db_dir: settings.db_dir.clone(),
            resolution_minutes: settings.resolution.num_minutes() as u32,
            issue_shortcuts: settings.issue_parser.shortcuts().clone(),
            breaks: settings.breaks.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct BreaksConfig {
    pub min_breaks_minutes: u32,
    pub min_work_time_minutes: u32,
    pub default_break: (Time, Time),
}

#[cfg(test)]
mod test {
    use crate::conf::{BreaksConfig, SettingsSer};
    use crate::data::JiraIssue;
    use crate::parsing::time::Time;
    use std::collections::BTreeMap;
    use std::path::{Path, PathBuf};
    use std::str::FromStr;

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
        };

        let pretty = serde_json::to_string_pretty(&orig).unwrap();

        let parsed = serde_json::from_str(&pretty).unwrap();
        assert_eq!(orig, parsed);
    }
}
