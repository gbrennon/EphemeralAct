use std::{
    fs::{create_dir_all, write},
    path::PathBuf,
};

use tempfile::TempDir;

/// Git repository fixture on disk, holding the workflow and action files a
/// scenario needs. The directory is removed when the fixture is dropped.
pub struct WorkflowRepository {
    _root: TempDir,
    name: String,
    canonical_path: PathBuf,
}

impl WorkflowRepository {
    pub fn named(name: &str) -> Self {
        let root = tempfile::tempdir().unwrap();
        let canonical_path = root.path().canonicalize().unwrap();
        let repository = Self {
            _root: root,
            name: name.into(),
            canonical_path,
        };
        let repo_path = repository.path();
        create_dir_all(&repo_path).unwrap();
        create_dir_all(repo_path.join(".git")).unwrap();
        repository
    }

    pub fn with_workflow(self, file_name: &str, body: &str) -> Self {
        let directory = self.path().join(".forgejo/workflows");
        create_dir_all(&directory).unwrap();
        write(directory.join(file_name), body).unwrap();
        self
    }

    pub fn with_action(self, action_path: &str, definition: &str) -> Self {
        let directory = self.path().join(action_path);
        create_dir_all(&directory).unwrap();
        write(directory.join("action.yml"), definition).unwrap();
        self
    }

    pub fn path(&self) -> PathBuf {
        self.canonical_path.join(&self.name)
    }

    pub fn path_argument(&self) -> String {
        self.path().display().to_string()
    }
}
