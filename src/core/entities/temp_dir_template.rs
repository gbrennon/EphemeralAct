use crate::core::value_objects::RepositoryName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TempDirTemplate(String);

impl TempDirTemplate {
    pub(crate) fn from_repo_name(name: &RepositoryName) -> Self {
        Self(format!("act-run-{}-XXXXXX", name.as_str()))
    }

    /// Returns the template string (e.g. `act-run-my-repo-XXXXXX`).
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
