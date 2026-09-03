#![allow(dead_code)]
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use ephact::{
    application::{
        dtos::{CollectActionFilesRequest, CollectActionFilesResponse},
        ports::{inbound::collect_action_files_port::CollectActionFilesPort, outbound::FileEntry},
    },
    domain::errors::StepError,
};

/// Returns a prepared set of files, recording the directories it walked.
#[derive(Clone)]
pub struct FakeCollectActionFilesPort {
    files: Vec<FileEntry>,
    failure: Option<String>,
    walked: Rc<RefCell<Vec<PathBuf>>>,
}

impl FakeCollectActionFilesPort {
    pub fn returning(files: Vec<FileEntry>) -> Self {
        Self {
            files,
            failure: None,
            walked: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn failing(message: &str) -> Self {
        Self {
            files: Vec::new(),
            failure: Some(message.to_string()),
            walked: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn walked(&self) -> Vec<PathBuf> {
        self.walked.borrow().clone()
    }
}

impl CollectActionFilesPort for FakeCollectActionFilesPort {
    fn execute(
        &self,
        request: CollectActionFilesRequest<'_>,
    ) -> Result<CollectActionFilesResponse, StepError> {
        self.walked
            .borrow_mut()
            .push(request.action_dir.to_path_buf());
        match &self.failure {
            Some(message) => Err(StepError::new(message.clone())),
            None => Ok(CollectActionFilesResponse {
                files: self.files.clone(),
            }),
        }
    }
}
