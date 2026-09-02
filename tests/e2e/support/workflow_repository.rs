use std::{
    fs::{create_dir_all, write},
    path::PathBuf,
};

use tempfile::TempDir;

/// Git repository fixture on disk, holding the workflow and action files a
/// scenario needs. The directory is removed when the fixture is dropped.
pub struct WorkflowRepository {
    root: TempDir,
    name: String,
}

impl WorkflowRepository {
    pub fn named(name: &str) -> Self {
        let root = tempfile::tempdir().unwrap();
        let repository = Self {
            root,
            name: name.into(),
        };
        create_dir_all(repository.path().join(".git")).unwrap();
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
        self.root.path().join(&self.name)
    }

    pub fn path_argument(&self) -> String {
        self.path().display().to_string()
    }
}
