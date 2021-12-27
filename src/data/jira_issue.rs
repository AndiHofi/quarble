use anyhow::bail;
use std::fmt::{Display, Formatter};
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
pub struct JiraIssue {
    pub ident: String,
    pub description: Option<String>,
    pub default_action: Option<String>,
}

impl JiraIssue {
    pub fn create(id: String) -> anyhow::Result<JiraIssue> {
        match id.split_once('-') {
            Some((project, number)) => {
                if !project.chars().all(|ch| ch.is_ascii_alphabetic()) {
                    bail!(
                        "Invalid Jira issue number, project ident is not ascii: {}",
                        project
                    );
                }

                if !number.chars().all(|ch| ch.is_ascii_digit()) {
                    bail!(
                        "Invalid Jira issue number, issue number is not numeric: {}",
                        number
                    );
                }
                Ok(JiraIssue {
                    ident: id.to_ascii_uppercase(),
                    description: None,
                    default_action: None,
                })
            }
            None => bail!("Invalid Jira issue number: {}", id),
        }
    }
}

impl Display for JiraIssue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.ident)?;
        if let Some(ref d) = self.description {
            write!(f, ": {}", d)?;
        }
        if let Some(ref da) = self.default_action {
            write!(f, " Default action: {}", da)?;
        }
        Ok(())
    }
}
