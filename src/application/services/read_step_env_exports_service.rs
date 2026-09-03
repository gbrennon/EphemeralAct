use std::collections::HashMap;

use crate::application::{
    constants::GITHUB_ENV_FILE, dtos::ReadStepEnvExportsRequest,
    ports::outbound::read_step_env_exports_port::ReadStepEnvExportsPort,
};

/// Service that reads the environment variables a step exported through
/// `GITHUB_ENV`.
///
/// Reading is best effort: a step that exported nothing never wrote the file,
/// so a failed read means no variables rather than a failure.
pub struct ReadStepEnvExportsService;

impl ReadStepEnvExportsService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ReadStepEnvExportsService {
    fn default() -> Self {
        Self::new()
    }
}

impl ReadStepEnvExportsPort for ReadStepEnvExportsService {
    fn execute(&self, request: ReadStepEnvExportsRequest<'_>) -> HashMap<String, String> {
        let mut exported = HashMap::new();
        if let Ok(output) = request.container.exec(
            &["cat".into(), GITHUB_ENV_FILE.into()],
            None,
            &HashMap::new(),
        ) {
            for line in output.stdout.lines() {
                let trimmed = line.trim();
                if let Some((key, value)) = trimmed.split_once('=') {
                    exported.insert(key.to_string(), value.to_string());
                }
            }
        }
        exported
    }
}
