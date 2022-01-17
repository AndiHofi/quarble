use crate::data::JiraIssue;
use crate::util::Timeline;

#[derive(Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct RecentIssuesData {
    pub issues: Vec<RecentIssue>,
}

impl RecentIssuesData {
    pub fn new() -> Self {
        Self { issues: Vec::new() }
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
    timeline: Timeline,
    max_len: usize,
}

impl RecentIssues {
    pub fn new(issues: RecentIssuesData, timeline: Timeline, max_len: usize) -> Self {
        let mut issues = issues.issues;
        issues.sort_by(|l, r| l.last_used.cmp(&r.last_used));

        Self {
            issues,
            timeline,
            max_len,
        }
    }

    pub fn issue_used(&mut self, issue: &JiraIssue) {
        let last_used = self.timeline.now();
        if let Some(recent) = self
            .issues
            .iter_mut()
            .find(|recent| recent.issue.ident == issue.ident)
        {
            recent.last_used = last_used;
            if let Some(description) = &issue.description {
                recent.issue.description = Some(description.clone());
            }
            if let Some(action) = &issue.default_action {
                recent.issue.default_action = Some(action.clone());
            }
        } else {
            self.issues.truncate(self.max_len);
            self.issues.insert(
                0,
                RecentIssue {
                    issue: issue.clone(),
                    last_used,
                },
            )
        }
    }

    pub fn list_recent(&self) -> &[RecentIssue] {
        self.issues.as_slice()
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
    use crate::data::recent_issues::vec_move_to_front;

    #[test]
    fn test_vec_move_to_front() {
        let mut v = vec![1];
        vec_move_to_front(&mut v, 0);
        assert_eq!(v.as_slice(), &[1]);

        v = vec![1, 2, 3, 4];
        vec_move_to_front(&mut v, 2);
        assert_eq!(v.as_slice(), &[3, 1, 2, 4]);
    }
}
