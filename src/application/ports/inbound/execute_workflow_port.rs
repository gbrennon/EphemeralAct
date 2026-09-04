use crate::application::dtos::{ExecuteWorkflowRequest, WorkflowExecution};

pub trait ExecuteWorkflowPort: Send + Sync {
    fn execute(
        &self,
        request: ExecuteWorkflowRequest<'_>,
    ) -> Result<WorkflowExecution, Box<dyn std::error::Error>>;
}
