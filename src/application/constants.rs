/// Directory the repository is mounted at inside the job container.
pub const CONTAINER_WORKSPACE: &str = "/workspace";

/// File the container writes `GITHUB_PATH` additions to.
pub const GITHUB_PATH_FILE: &str = "/workspace/.github_path";

/// File the container writes `GITHUB_ENV` additions to.
pub const GITHUB_ENV_FILE: &str = "/workspace/.github_env";

/// Search order for workflow directories, so a Forgejo repository is detected
/// before falling back to the GitHub layout.
pub const WORKFLOW_DIRECTORIES: [&str; 2] = [".forgejo/workflows", ".github/workflows"];
