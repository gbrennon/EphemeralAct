/// Absolute path the repository is mounted at inside a job container.
pub const CONTAINER_WORKSPACE: &str = "/workspace";

/// In-container file steps append `KEY=value` lines to, mirroring `$GITHUB_ENV`.
pub const GITHUB_ENV_FILE: &str = "/workspace/.github_env";

/// In-container file steps append directories to, mirroring `$GITHUB_PATH`.
pub const GITHUB_PATH_FILE: &str = "/workspace/.github_path";
