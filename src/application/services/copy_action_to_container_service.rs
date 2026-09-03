use std::{collections::HashMap, path::Path};

use crate::{
    application::{
        dtos::{CollectActionFilesRequest, CopyActionToContainerRequest},
        ports::inbound::{
            collect_action_files_port::CollectActionFilesPort,
            copy_action_to_container_port::CopyActionToContainerPort,
        },
    },
    domain::errors::StepError,
};

/// Directory inside the container that holds actions copied in for a run.
const CONTAINER_ACTIONS_ROOT: &str = "/tmp/ephemeral-act-actions";

/// Service that copies an action's files into the container that runs it.
pub struct CopyActionToContainerService {
    file_collector: Box<dyn CollectActionFilesPort>,
}

impl CopyActionToContainerService {
    pub fn new(file_collector: Box<dyn CollectActionFilesPort>) -> Self {
        Self { file_collector }
    }

    /// Names the container-side directory an action is copied to.
    fn container_action_dir(action_dir: &Path) -> String {
        let slug: String = action_dir
            .display()
            .to_string()
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || character == '.' || character == '-' {
                    character
                } else {
                    '_'
                }
            })
            .collect();
        format!("{CONTAINER_ACTIONS_ROOT}/{slug}")
    }
}

impl CopyActionToContainerPort for CopyActionToContainerService {
    fn execute(&self, request: CopyActionToContainerRequest<'_>) -> Result<String, StepError> {
        let container_dir = Self::container_action_dir(request.action_dir);
        let files = self
            .file_collector
            .execute(CollectActionFilesRequest {
                action_dir: request.action_dir,
            })?
            .files;

        request
            .container
            .exec(
                &["mkdir".into(), "-p".into(), container_dir.clone()],
                None,
                &HashMap::new(),
            )
            .map_err(|error| {
                StepError::new(format!("failed to create action directory: {error:?}"))
            })?;
        request
            .container
            .copy_to(&container_dir, &files)
            .map_err(|error| StepError::new(format!("failed to copy action files: {error:?}")))?;

        Ok(container_dir)
    }
}
