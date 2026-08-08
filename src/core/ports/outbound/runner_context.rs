use std::collections::HashMap;

/// Context describing the runner environment inside the container.
#[derive(Debug, Clone)]
pub struct RunnerContext {
    /// Workspace directory path
    pub workspace: String,
    /// Home directory path
    pub home: String,
    /// GitHub Actions action path
    pub action_path: String,
    /// Temp directory path
    pub temp: String,
    /// Tool cache directory path
    pub tool_cache: String,
    /// Environment variables visible inside the container
    pub env: HashMap<String, String>,
}
