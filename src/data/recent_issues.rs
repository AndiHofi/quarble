use arc_swap::{ArcSwap, Guard};
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::sync::Arc;

use crate::conf::SettingsRef;
use crate::data::JiraIssue;
use crate::util::update_arcswap;

#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RecentIssuesData {
    pub issues: Vec<RecentIssue>,
}

#[derive(Clone)]
pub struct RecentIssuesRef(Arc<ArcSwap<RecentIssues>>);

impl Debug for RecentIssuesRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let d = self.0.load();
        <RecentIssues as Debug>::fmt(d.deref().deref(), f)
    }
}

impl RecentIssuesRef {
    pub fn new(inner: RecentIssues) -> Self {
        Self(Arc::new(ArcSwap::new(Arc::new(inner))))
    }

    pub fn empty(settings: SettingsRef) -> Self {
        Self::new(RecentIssues::new(Default::default(), settings))
    }

    pub fn issue_used_with_comment(&self, issue: &JiraIssue, comment: Option<&str>) {
        update_arcswap(&self.0, |r: &mut RecentIssues| {
            r.issue_used_with_comment(issue, comment)
        })
    }

    pub fn borrow(&self) -> Guard<Arc<RecentIssues>> {
        self.0.load()
    }

    #[cfg(test)]
    pub fn get(&self, index: usize) -> RecentIssue {
        self.borrow().issues[index].clone()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RecentIssue {
    pub last_used: chrono::NaiveDateTime,
    pub issue: JiraIssue,
}

#[derive(Clone, Debug)]
pub struct RecentIssues {
    issues: Vec<RecentIssue>,
    settings: SettingsRef,
    max_len: NonZeroUsize,
}

impl RecentIssues {
    pub fn new(issues: RecentIssuesData, settings: SettingsRef) -> Self {
        let max_len = settings.load().max_recent_issues;
        let max_len = NonZeroUsize::new(if max_len < 1 { 1 } else { max_len }).unwrap();
        let guard = settings.load();
        let shortcuts = guard.issue_parser.shortcuts();

        let issues_sorted =
            BTreeMap::from_iter(issues.issues.into_iter().map(|e| (e.last_used, e)));

        let issues = issues_sorted
            .into_values()
            .rev()
            .filter(|r| !shortcuts.values().any(|sc| sc.ident == r.issue.ident))
            .take(max_len.get())
            .map(normalize_recent)
            .collect();

        Self {
            issues,
            settings,
            max_len,
        }
    }

    pub fn issue_used(&mut self, issue: &JiraIssue) {
        if self.is_shortcut(issue) {
            return;
        }

        let last_used = self.settings.load().timeline.now();

        if let Some(recent) =
            find_and_move_to_front(&mut self.issues, |i| i.issue.ident == issue.ident)
        {
            recent.last_used = last_used;
            update_string(&mut recent.issue.description, issue.description.as_deref());
            update_string(
                &mut recent.issue.default_action,
                issue.default_action.as_deref(),
            );
        } else {
            self.issues.truncate(self.max_len.get() - 1);
            self.issues.insert(
                0,
                RecentIssue {
                    issue: issue.clone(),
                    last_used,
                },
            )
        }
    }

    pub fn issue_used_with_comment(&mut self, issue: &JiraIssue, comment: Option<&str>) {
        match comment {
            Some(c) => {
                let mut issue = issue.clone();
                issue.default_action = Some(c.to_string());
                self.issue_used(&issue)
            }
            None => self.issue_used(issue),
        }
    }

    pub fn list_recent(&self) -> &[RecentIssue] {
        self.issues.as_slice()
    }

    pub fn find_recent(&self, num: usize) -> Option<&RecentIssue> {
        self.issues.get(num)
    }

    fn is_shortcut(&self, issue: &JiraIssue) -> bool {
        let guard = self.settings.load();
        guard
            .issue_parser
            .shortcuts()
            .values()
            .any(|sc| sc.ident == issue.ident)
    }
}

fn normalize_recent(mut recent: RecentIssue) -> RecentIssue {
    empty_to_none(&mut recent.issue.default_action);
    empty_to_none(&mut recent.issue.description);
    recent
}

fn empty_to_none(str: &mut Option<String>) {
    if matches!(str.as_deref(), Some(s) if s.trim().is_empty()) {
        *str = None;
    }
}

fn update_string(target: &mut Option<String>, source: Option<&str>) {
    match source {
        Some(source) if !source.trim().is_empty() && Some(source) != target.as_deref() => {
            *target = Some(source.trim().to_string())
        }
        _ => (),
    }
}

fn vec_move_to_front<T>(v: &mut [T], to_move: usize) {
    let to_rotate = &mut v[0..=to_move];
    to_rotate.rotate_right(1);
}

fn find_and_move_to_front<T>(v: &mut [T], mut f: impl FnMut(&T) -> bool) -> Option<&mut T> {
    if let Some(index) = v
        .iter_mut()
        .enumerate()
        .find(|(_, b)| f(*b))
        .map(|(i, _)| i)
    {
        vec_move_to_front(v, index);
        Some(&mut v[0])
    } else {
        None
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeMap;
    use std::sync::Arc;

    use chrono::NaiveDateTime;

    use crate::conf::{into_settings_ref, Settings};
    use crate::data::recent_issues::vec_move_to_front;
    use crate::data::{JiraIssue, RecentIssue, RecentIssues, RecentIssuesData};
    use crate::parsing::JiraIssueParser;
    use crate::util::{StaticTimeline, TimelineProvider};

    impl RecentIssues {
        fn list_recent_view(&self) -> Vec<&RecentIssue> {
            self.issues.iter().collect()
        }
    }

    #[test]
    fn test_vec_move_to_front() {
        let mut v = vec![1];
        vec_move_to_front(&mut v, 0);
        assert_eq!(v.as_slice(), &[1]);

        v = vec![1, 2, 3, 4];
        vec_move_to_front(&mut v, 2);
        assert_eq!(v.as_slice(), &[3, 1, 2, 4]);
    }

    #[test]
    fn sorts_on_init_and_truncates() {
        let timeline = Arc::new(StaticTimeline::parse("2022-01-10 12:00"));
        let settings = into_settings_ref(Settings {
            timeline: timeline.clone(),
            max_recent_issues: 3,
            ..Default::default()
        });

        let recent1 = next_recent(&timeline, "i1");
        let recent2 = next_recent(&timeline, "i2");
        let recent3 = next_recent(&timeline, "i3");
        let recent4 = next_recent(&timeline, "i4");

        let recent = RecentIssues::new(
            RecentIssuesData {
                issues: [&recent2, &recent1, &recent3, &recent4]
                    .into_iter()
                    .cloned()
                    .collect(),
            },
            settings,
        );

        assert_eq!(
            recent.list_recent_view(),
            vec![&recent4, &recent3, &recent2]
        );
    }

    #[test]
    fn ignores_shortcuts() {
        let timeline = Arc::new(StaticTimeline::parse("2022-01-10 12:00"));
        let settings = Settings {
            issue_parser: JiraIssueParser::new(BTreeMap::from_iter([
                ('a', issue("a1")),
                ('b', issue("a2")),
            ])),
            timeline: timeline.clone(),
            max_recent_issues: 3,
            ..Default::default()
        };

        let settings = into_settings_ref(settings);

        let recent1 = next_recent(&timeline, "i1");
        let recent2 = next_recent(&timeline, "i2");
        let recent3 = next_recent(&timeline, "a1");
        let recent4 = next_recent(&timeline, "a2");

        let mut recent = RecentIssues::new(
            RecentIssuesData {
                issues: [&recent2, &recent1, &recent3, &recent4]
                    .into_iter()
                    .cloned()
                    .collect(),
            },
            settings,
        );

        assert_eq!(recent.list_recent_view(), vec![&recent2, &recent1]);

        let recent5 = next_recent(&timeline, "a1");
        recent.issue_used(&recent5.issue);

        assert_eq!(recent.list_recent_view(), vec![&recent2, &recent1]);

        let recent6 = next_recent(&timeline, "i3");
        recent.issue_used(&recent6.issue);
        assert_eq!(recent.list_recent(), vec![recent6, recent2, recent1]);
    }

    #[test]
    fn test_stays_ordered() {
        let timeline = Arc::new(StaticTimeline::parse("2022-01-10 12:00"));
        let settings = into_settings_ref(Settings {
            timeline: timeline.clone(),
            max_recent_issues: 5,
            ..Default::default()
        });

        let mut recent = RecentIssues::new(RecentIssuesData::default(), settings);

        let recent1 = next_recent(&timeline, "i1");
        recent.issue_used(&recent1.issue);
        let recent2 = next_recent(&timeline, "i2");
        recent.issue_used(&recent2.issue);
        let recent3 = next_recent(&timeline, "i3");
        recent.issue_used(&recent3.issue);

        assert_eq!(
            recent.list_recent(),
            &[recent3.clone(), recent2.clone(), recent1.clone()]
        );

        let recent1_1 = next_recent(&timeline, "i1");
        recent.issue_used(&recent1_1.issue);

        assert_eq!(
            recent.list_recent(),
            &[recent1_1.clone(), recent3.clone(), recent2.clone()]
        );

        let recent1_2 = next_recent(&timeline, "i1");
        recent.issue_used(&recent1_2.issue);
        assert_eq!(
            recent.list_recent(),
            &[recent1_2.clone(), recent3.clone(), recent2.clone()]
        );

        let recent4 = next_recent(&timeline, "i4");
        recent.issue_used(&recent4.issue);

        let recent5 = next_recent(&timeline, "i5");
        recent.issue_used(&recent5.issue);

        assert_eq!(
            recent.list_recent(),
            &[
                recent5.clone(),
                recent4.clone(),
                recent1_2.clone(),
                recent3.clone(),
                recent2.clone()
            ]
        );

        let recent6 = next_recent(&timeline, "i6");
        recent.issue_used(&recent6.issue);

        assert_eq!(
            recent.list_recent(),
            &[
                recent6.clone(),
                recent5.clone(),
                recent4.clone(),
                recent1_2.clone(),
                recent3.clone(),
            ]
        );
    }

    fn issue(issue: &str) -> JiraIssue {
        JiraIssue {
            ident: issue.to_string(),
            description: None,
            default_action: None,
        }
    }

    fn recent(time: NaiveDateTime, issue: &str) -> RecentIssue {
        RecentIssue {
            last_used: time,
            issue: JiraIssue {
                ident: issue.to_string(),
                description: None,
                default_action: None,
            },
        }
    }

    fn next_recent(timeline: &StaticTimeline, issue: &str) -> RecentIssue {
        timeline.advance();
        recent(timeline.now(), issue)
    }
}
