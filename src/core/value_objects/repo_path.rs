use std::path::{Path, PathBuf};

use crate::core::errors::CoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoPath(PathBuf);

impl RepoPath {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, CoreError> {
        let path: PathBuf = path.into();

        let canonical = path
            .canonicalize()
            .map_err(|e| CoreError::InvalidRepositoryPath(format!(
                "cannot resolve '{}': {}",
                path.display(),
                e
            )))?;

        if !canonical.join(".git").exists() {
            return Err(CoreError::NotAGitRepository(
                canonical.display().to_string(),
            ));
        }

        Ok(Self(canonical))
    }

    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn new_with_valid_git_repo_succeeds() {
        let crate_root = env::var("CARGO_MANIFEST_DIR").unwrap();
        let repo_path = RepoPath::new(&crate_root).unwrap();
        assert!(repo_path.as_path().join(".git").exists());
    }

    #[test]
    fn new_with_nonexistent_path_returns_invalid_repository_path() {
        let result = RepoPath::new("/nonexistent/path/12345");
        assert!(matches!(result, Err(CoreError::InvalidRepositoryPath(_))));
    }

    #[test]
    fn new_with_non_repo_dir_returns_not_a_git_repository() {
        let tmp = env::temp_dir();
        let result = RepoPath::new(&tmp);
        assert!(matches!(result, Err(CoreError::NotAGitRepository(_))));
    }

    #[test]
    fn as_path_returns_canonical_path() {
        let crate_root = env::var("CARGO_MANIFEST_DIR").unwrap();
        let expected = std::fs::canonicalize(&crate_root).unwrap();

        let repo_path = RepoPath::new(&crate_root).unwrap();
        assert_eq!(repo_path.as_path(), expected.as_path());
    }
}