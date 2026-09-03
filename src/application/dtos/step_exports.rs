use std::collections::HashMap;

/// What a step exported to the steps that follow it.
pub struct StepExports {
    /// Directories the step added through `GITHUB_PATH`.
    pub path_additions: Vec<String>,
    /// Environment variables the step added through `GITHUB_ENV`.
    pub env: HashMap<String, String>,
}
