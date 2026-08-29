/// Core domain errors that can occur during workflow execution setup and configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreError {
    /// The provided repository path is invalid or does not exist.
    InvalidRepositoryPath(String),

    /// The provided path is not a git repository (no .git directory found).
    NotAGitRepository(String),

    /// A repository name was required but an empty string was provided.
    EmptyRepositoryName,

    /// An unknown container engine was specified (only "podman" or "docker" are supported).
    UnknownContainerEngine(String),
}
