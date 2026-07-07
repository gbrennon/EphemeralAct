use std::path::{Path, PathBuf};

use crate::core::errors::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitDirKind {
    Standalone,
    Worktree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoPath {
    path: PathBuf,
    git_dir_kind: GitDirKind,
}

impl RepoPath {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, CoreError> {
        let path: PathBuf = path.into();

        if !path.is_dir() {
            return Err(CoreError::InvalidRepositoryPath(format!(
                "'{}' is not a directory",
                path.display()
            )));
        }

        let canonical = path
            .canonicalize()
            .map_err(|e| CoreError::InvalidRepositoryPath(format!(
                "cannot resolve '{}': {}",
                path.display(),
                e
            )))?;

        let git_dir = canonical.join(".git");
        let git_dir_kind = if git_dir.is_dir() {
            GitDirKind::Standalone
        } else if git_dir.is_file() {
            GitDirKind::Worktree
        } else {
            return Err(CoreError::NotAGitRepository(
                canonical.display().to_string(),
            ));
        };

        Ok(Self {
            path: canonical,
            git_dir_kind,
        })
    }

    pub fn as_path(&self) -> &Path {
        &self.path
    }

    pub fn git_dir_kind(&self) -> GitDirKind {
        self.git_dir_kind
    }

    pub fn is_standalone(&self) -> bool {
        self.git_dir_kind == GitDirKind::Standalone
    }

    pub fn is_worktree(&self) -> bool {
        self.git_dir_kind == GitDirKind::Worktree
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
    fn new_with_file_instead_of_dir_returns_invalid_repository_path() {
        let cargo_toml = env::var("CARGO_MANIFEST_DIR").unwrap() + "/Cargo.toml";
        let result = RepoPath::new(cargo_toml);
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

    #[test]
    fn git_dir_kind_identifies_repo_type() {
        let crate_root = env::var("CARGO_MANIFEST_DIR").unwrap();
        let repo_path = RepoPath::new(&crate_root).unwrap();
        let kind = repo_path.git_dir_kind();
        assert!(kind == GitDirKind::Standalone || kind == GitDirKind::Worktree);
    }
}