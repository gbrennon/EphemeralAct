use serde::Deserialize;

/// Default run settings for shell and working directory.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RunDefaults {
    /// Default shell for run steps.
    pub shell: Option<String>,

    /// Default working directory for run steps.
    #[serde(rename = "working-directory")]
    pub working_directory: Option<String>,
}
