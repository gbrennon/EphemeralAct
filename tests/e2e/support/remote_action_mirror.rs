use std::{fs::write, path::PathBuf};

use tempfile::TempDir;

/// Checkout of a remote action as an action fetcher would leave it on disk,
/// standing in for a clone from a forge.
pub struct RemoteActionMirror {
    root: TempDir,
}

impl RemoteActionMirror {
    pub fn new() -> Self {
        Self {
            root: tempfile::tempdir().unwrap(),
        }
    }

    pub fn with_definition(self, definition: &str) -> Self {
        write(self.path().join("action.yml"), definition).unwrap();
        self
    }

    pub fn with_file(self, file_name: &str, body: &str) -> Self {
        write(self.path().join(file_name), body).unwrap();
        self
    }

    pub fn path(&self) -> PathBuf {
        self.root.path().to_path_buf()
    }
}
