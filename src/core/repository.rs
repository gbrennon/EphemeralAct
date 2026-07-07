use crate::core::value_objects::{RepoPath, RepositoryName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    path: RepoPath,
    name: RepositoryName,
}

impl Repository {
    /// Creates a repository from a validated path and name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::{RepoPath, RepositoryName};
    /// # use ephemeral_act::core::Repository;
    /// # use std::env;
    /// # let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    /// let path = RepoPath::new(dir).unwrap();
    /// let name = RepositoryName::new("my-repo".into()).unwrap();
    /// let repo = Repository::new(path, name);
    /// ```
    pub fn new(path: RepoPath, name: RepositoryName) -> Self {
        Self { path, name }
    }

    /// Returns the repository's canonical path.
    pub fn path(&self) -> &RepoPath {
        &self.path
    }

    /// Returns the repository's name.
    pub fn name(&self) -> &RepositoryName {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn repo_path() -> RepoPath {
        let crate_root = env::var("CARGO_MANIFEST_DIR").unwrap();
        RepoPath::new(crate_root).unwrap()
    }

    fn repo_name() -> RepositoryName {
        RepositoryName::new("test-repo".into()).unwrap()
    }

    #[test]
    fn new_sets_provided_path_and_name() {
        let path = repo_path();
        let name = repo_name();

        let repo = Repository::new(path.clone(), name.clone());

        assert_eq!(repo.path(), &path);
        assert_eq!(repo.name(), &name);
    }
}
