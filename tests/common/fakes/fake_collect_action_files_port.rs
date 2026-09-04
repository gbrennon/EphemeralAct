#![allow(dead_code)]
use parking_lot::Mutex;
use std::{path::PathBuf, sync::Arc};

use ephact::{
    application::dtos::{CollectActionFilesRequest, CollectActionFilesResponse, FileEntry},
    domain::errors::StepError,
    infrastructure::actions::collect_action_files_port::CollectActionFilesPort,
};

/// Returns a prepared set of files, recording the directories it walked.
#[derive(Clone)]
pub struct FakeCollectActionFilesPort {
    files: Vec<FileEntry>,
    failure: Option<String>,
    walked: Arc<Mutex<Vec<PathBuf>>>,
}

impl FakeCollectActionFilesPort {
    pub fn returning(files: Vec<FileEntry>) -> Self {
        Self {
            files,
            failure: None,
            walked: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            files: Vec::new(),
            failure: Some(message.to_string()),
            walked: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn walked(&self) -> Vec<PathBuf> {
        self.walked.lock().clone()
    }
}

impl CollectActionFilesPort for FakeCollectActionFilesPort {
    fn execute(
        &self,
        request: CollectActionFilesRequest<'_>,
    ) -> Result<CollectActionFilesResponse, StepError> {
        self.walked.lock().push(request.action_dir.to_path_buf());
        match &self.failure {
            Some(message) => Err(StepError::new(message.clone())),
            None => Ok(CollectActionFilesResponse {
                files: self.files.clone(),
            }),
        }
    }
}
