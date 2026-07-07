use crate::core::value_objects::{RepoPath, RepositoryId, RepositoryName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Repository {
    pub id: RepositoryId,
    pub name: RepositoryName,
    pub path: RepoPath,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Repository {
    pub fn new(path: RepoPath, name: RepositoryName) -> Self {
        Self {
            id: RepositoryId::new(),
            name,
            path,
            created_at: chrono::Utc::now(),
        }
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

        assert_eq!(repo.path, path);
        assert_eq!(repo.name, name);
    }

    #[test]
    fn new_generates_unique_ids() {
        let repo1 = Repository::new(repo_path(), repo_name());
        let repo2 = Repository::new(repo_path(), repo_name());

        assert_ne!(repo1.id, repo2.id);
    }

    #[test]
    fn new_sets_created_at() {
        let before = chrono::Utc::now();
        let repo = Repository::new(repo_path(), repo_name());
        let after = chrono::Utc::now();

        assert!(repo.created_at >= before);
        assert!(repo.created_at <= after);
    }
}