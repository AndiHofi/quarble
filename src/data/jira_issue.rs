use anyhow::bail;
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct JiraIssue {
    ident: String,
}

impl JiraIssue {
    pub fn create(id: String) -> anyhow::Result<JiraIssue> {
        match id.split_once('-') {
            Some((project, number)) => {
                if !project.chars().all(|ch| ch.is_ascii_alphabetic()) {
                    bail!("Invalid Jira issue number, project ident is not ascii: {}", project);
                }

                if !number.chars().all(|ch| ch.is_ascii_digit()) {
                    bail!("Invalid Jira issue number, issue number is not numeric: {}", number);
                }
                Ok(JiraIssue { ident: id.to_ascii_uppercase() })
            }
            None => bail!("Invalid Jira issue number: {}", id)
        }
    }
}