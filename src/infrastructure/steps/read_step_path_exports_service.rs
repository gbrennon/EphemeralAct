use crate::infrastructure::steps::read_step_path_exports_port::ReadStepPathExportsPort;
use std::collections::HashMap;

use crate::{
    application::dtos::ReadStepPathExportsRequest,
    infrastructure::containers::workspace::GITHUB_PATH_FILE,
};

/// Service that reads the directories a step exported through `GITHUB_PATH`.
///
/// Reading is best effort: a step that exported nothing never wrote the file,
/// so a failed read means no additions rather than a failure.
pub struct ReadStepPathExportsService;

impl ReadStepPathExportsService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReadStepPathExportsService {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadStepPathExportsPort for ReadStepPathExportsService {
    fn execute(&self, request: ReadStepPathExportsRequest<'_>) -> Vec<String> {
        let mut additions = Vec::new();
        if let Ok(output) = request.container.exec(
            &["cat".into(), GITHUB_PATH_FILE.into()],
            None,
            &HashMap::new(),
        ) {
            for line in output.stdout.lines() {
                let trimmed = line.trim();
                if !trimmed.is_empty() {
                    additions.push(trimmed.to_string());
                }
            }
        }
        additions
    }
}
