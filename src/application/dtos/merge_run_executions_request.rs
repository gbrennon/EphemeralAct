use crate::application::dtos::WorkflowExecution;

/// Request DTO for the
/// [`MergeRunExecutionsPort`](crate::application::ports::outbound::merge_run_executions_port::MergeRunExecutionsPort)
/// inbound port.
pub struct MergeRunExecutionsRequest {
    /// Executions to merge, in the order they ran.
    pub executions: Vec<WorkflowExecution>,
    /// Whether the run covered every workflow of the repository.
    pub all_workflows: bool,
}
