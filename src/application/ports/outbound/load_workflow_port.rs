use crate::{application::dtos::LoadWorkflowRequest, domain::workflow::Workflow};

/// Inbound port for reading and parsing a workflow file.
pub trait LoadWorkflowPort {
    /// Reads the workflow file and parses it.
    fn execute(
        &self,
        request: LoadWorkflowRequest<'_>,
    ) -> Result<Workflow, Box<dyn std::error::Error>>;
}
