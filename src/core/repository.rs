use crate::core::value_objects::{RepoPath, RepositoryName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    path: RepoPath,
    name: RepositoryName,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl Repository {
    pub fn new(path: RepoPath, name: RepositoryName) -> Self {
        Self {
            path,
            name,
            created_at: chrono::Utc::now(),
        }
    }

    pub fn path(&self) -> &RepoPath {
        &self.path
    }

    pub fn name(&self) -> &RepositoryName {
        &self.name
    }

    pub fn created_at(&self) -> &chrono::DateTime<chrono::Utc> {
        &self.created_at
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

    #[test]
    fn new_sets_created_at() {
        let before = chrono::Utc::now();
        let repo = Repository::new(repo_path(), repo_name());
        let after = chrono::Utc::now();

        assert!(repo.created_at() >= &before);
        assert!(repo.created_at() <= &after);
    }
}