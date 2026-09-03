use crate::{application::dtos::CopyActionToContainerRequest, domain::errors::StepError};

/// Inbound port for copying an action's files into the job's container.
pub trait CopyActionToContainerPort {
    /// Copies the action in and returns the container-side directory.
    fn execute(&self, request: CopyActionToContainerRequest<'_>) -> Result<String, StepError>;
}
