use std::fmt;

use crate::core::errors::CoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryName(String);

impl RepositoryName {
    /// Creates a repository name from a string.
    ///
    /// Returns [`CoreError::EmptyRepositoryName`] if the string is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::RepositoryName;
    /// let name = RepositoryName::new("my-repo".into()).unwrap();
    /// assert_eq!(name.as_str(), "my-repo");
    /// ```
    pub fn new(name: String) -> Result<Self, CoreError> {
        if name.is_empty() {
            Err(CoreError::EmptyRepositoryName)
        } else {
            Ok(Self(name))
        }
    }

    /// Derives a repository name from the final component of a [`RepoPath`].
    ///
    /// Returns [`CoreError::EmptyRepositoryName`] if the path has no file name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::{RepoPath, RepositoryName};
    /// # use std::env;
    /// # let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    /// let repo_path = RepoPath::new(dir).unwrap();
    /// let name = RepositoryName::from_repo_path(&repo_path).unwrap();
    /// assert!(!name.as_str().is_empty());
    /// ```
    pub fn from_repo_path(repo_path: &super::repo_path::RepoPath) -> Result<Self, CoreError> {
        let name = repo_path
            .as_path()
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        Self::new(name)
    }

    /// Returns the repository name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RepositoryName {
    /// Formats the repository name using its inner string.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    use super::super::repo_path::RepoPath;

    #[test]
    fn new_with_valid_name_succeeds() {
        let name = RepositoryName::new("my-repo".into()).unwrap();
        assert_eq!(name.as_str(), "my-repo");
    }

    #[test]
    fn new_with_empty_name_returns_empty_repository_name() {
        let result = RepositoryName::new("".into());
        assert_eq!(result, Err(CoreError::EmptyRepositoryName));
    }

    #[test]
    fn from_repo_path_derives_name_from_path() {
        let crate_root = env::var("CARGO_MANIFEST_DIR").unwrap();
        let repo_path = RepoPath::new(crate_root).unwrap();
        let name = RepositoryName::from_repo_path(&repo_path).unwrap();
        assert!(!name.as_str().is_empty());
    }

    #[test]
    fn display_formats_inner_string() {
        let name = RepositoryName::new("my-repo".into()).unwrap();
        assert_eq!(format!("{}", name), "my-repo");
    }

    #[test]
    fn as_str_returns_inner() {
        let name = RepositoryName::new("my-repo".into()).unwrap();
        assert_eq!(name.as_str(), "my-repo");
    }
}
