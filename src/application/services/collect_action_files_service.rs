use std::{
    fs::{read, read_dir},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use crate::{
    application::{
        dtos::{CollectActionFilesRequest, CollectActionFilesResponse},
        ports::{inbound::collect_action_files_port::CollectActionFilesPort, outbound::FileEntry},
    },
    domain::errors::StepError,
};

/// Directory never copied into the container along with an action.
const GIT_DIRECTORY: &str = ".git";

/// Service that reads every file making up an action, so it can be copied into
/// the container that runs it.
pub struct CollectActionFilesService;

impl CollectActionFilesService {
    pub fn new() -> Self {
        Self
    }

    /// Walks `directory`, reading each file it holds into `files`.
    fn collect_files_into(
        root: &Path,
        directory: &Path,
        files: &mut Vec<FileEntry>,
    ) -> Result<(), StepError> {
        let listing = read_dir(directory).map_err(|error| {
            StepError::new(format!(
                "failed to read action directory {}: {error}",
                directory.display()
            ))
        })?;

        for entry in listing {
            let path: PathBuf = entry
                .map_err(|error| StepError::new(format!("failed to read action entry: {error}")))?
                .path();

            if path.file_name().is_some_and(|name| name == GIT_DIRECTORY) {
                continue;
            }
            if path.is_dir() {
                Self::collect_files_into(root, &path, files)?;
                continue;
            }

            let relative = path
                .strip_prefix(root)
                .map_err(|error| StepError::new(format!("action file outside action: {error}")))?;
            let content = read(&path).map_err(|error| {
                StepError::new(format!("failed to read {}: {error}", path.display()))
            })?;
            let mode = path
                .metadata()
                .map(|metadata| metadata.permissions().mode() & 0o7777)
                .unwrap_or(0o644);

            files.push(FileEntry {
                path: relative.display().to_string(),
                content,
                mode,
            });
        }

        Ok(())
    }
}

impl Default for CollectActionFilesService {
    fn default() -> Self {
        Self::new()
    }
}

impl CollectActionFilesPort for CollectActionFilesService {
    fn execute(
        &self,
        request: CollectActionFilesRequest<'_>,
    ) -> Result<CollectActionFilesResponse, StepError> {
        let mut files = Vec::new();
        Self::collect_files_into(request.action_dir, request.action_dir, &mut files)?;
        Ok(CollectActionFilesResponse { files })
    }
}
