use crate::core::Repository;
use crate::core::value_objects::{CleanupPolicy, RepositoryName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TempDirTemplate(String);

impl TempDirTemplate {
    fn from_repo_name(name: &RepositoryName) -> Self {
        Self(format!("act-run-{}-XXXXXX", name.as_str()))
    }

    /// Returns the template string (e.g. `act-run-my-repo-XXXXXX`).
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EphemeralRepository {
    temp_dir_template: TempDirTemplate,
    needs_standalone_conversion: bool,
    cleanup_policy: CleanupPolicy,
}

impl EphemeralRepository {
    /// Creates an ephemeral repository descriptor from a source repository.
    ///
    /// Detects whether the source is a worktree and sets
    /// [`needs_standalone_conversion`](Self::needs_standalone_conversion) accordingly.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::{CleanupPolicy, RepoPath, RepositoryName};
    /// # use ephemeral_act::core::{EphemeralRepository, Repository};
    /// # use std::env;
    /// # let dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    /// let source = Repository::new(
    ///     RepoPath::new(dir).unwrap(),
    ///     RepositoryName::new("my-repo".into()).unwrap(),
    /// );
    /// let ephemeral = EphemeralRepository::new(&source, CleanupPolicy::CleanupOnExit);
    /// ```
    pub fn new(source: &Repository, cleanup_policy: CleanupPolicy) -> Self {
        let needs_standalone_conversion = source.path().is_worktree();

        Self {
            temp_dir_template: TempDirTemplate::from_repo_name(source.name()),
            needs_standalone_conversion,
            cleanup_policy,
        }
    }

    /// Returns the temp directory template.
    pub fn temp_dir_template(&self) -> &TempDirTemplate {
        &self.temp_dir_template
    }

    /// Returns `true` if the source is a worktree and needs conversion to a standalone repo.
    pub fn needs_standalone_conversion(&self) -> bool {
        self.needs_standalone_conversion
    }

    /// Returns the cleanup policy for this ephemeral repository.
    pub fn cleanup_policy(&self) -> &CleanupPolicy {
        &self.cleanup_policy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::value_objects::RepoPath;
    use std::env;

    fn source_repo() -> Repository {
        let crate_root = env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = RepoPath::new(crate_root).unwrap();
        let name = RepositoryName::from_repo_path(&path).unwrap();
        Repository::new(path, name)
    }

    #[test]
    fn new_sets_temp_dir_template_from_repo_name() {
        let source = source_repo();
        let ephemeral = EphemeralRepository::new(&source, CleanupPolicy::CleanupOnExit);

        assert!(
            ephemeral
                .temp_dir_template()
                .as_str()
                .starts_with("act-run-")
        );
    }

    #[test]
    fn new_sets_cleanup_policy() {
        let source = source_repo();
        let ephemeral = EphemeralRepository::new(&source, CleanupPolicy::Preserve);

        assert_eq!(ephemeral.cleanup_policy(), &CleanupPolicy::Preserve);
    }

    #[test]
    fn needs_standalone_conversion_reflects_source_worktree_status() {
        let source = source_repo();
        let ephemeral = EphemeralRepository::new(&source, CleanupPolicy::CleanupOnExit);

        // Our test repo is a worktree, so this should be true
        let is_worktree = source.path().is_worktree();
        assert_eq!(ephemeral.needs_standalone_conversion(), is_worktree);
    }
}
