use crate::application::dtos::{ExecuteWorkflowRequest, WorkflowExecution};

/// Inbound port for running every job of one workflow file.
pub trait ExecuteWorkflowPort {
    /// Plans the workflow's jobs and runs them in dependency order.
    fn execute(
        &self,
        request: ExecuteWorkflowRequest<'_>,
    ) -> Result<WorkflowExecution, Box<dyn std::error::Error>>;
}
