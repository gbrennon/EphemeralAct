use serde::Deserialize;

use super::RunDefaults;

/// Default settings for all jobs in a workflow.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Defaults {
    /// Default run settings (shell, working-directory).
    pub run: Option<RunDefaults>,
}
