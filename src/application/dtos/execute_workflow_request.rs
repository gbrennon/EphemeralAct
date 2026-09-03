use std::path::Path;

use crate::domain::expression::EvalContext;

/// Request DTO for the
/// [`ExecuteWorkflowPort`](crate::application::ports::outbound::execute_workflow_port::ExecuteWorkflowPort)
/// inbound port.
pub struct ExecuteWorkflowRequest<'a> {
    /// Workflow file whose jobs are executed.
    pub workflow_file: &'a Path,
    /// Repository directory the run executes against.
    pub repo_path: &'a Path,
    /// Context the workflow's steps are evaluated against.
    pub context: &'a EvalContext,
}
