use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
pub enum CoreError {
    #[error("invalid repository path: {0}")]
    InvalidRepositoryPath(String),

    #[error("'{0}' is not a git repository (no .git found)")]
    NotAGitRepository(String),

    #[error("repository name must not be empty")]
    EmptyRepositoryName,
}