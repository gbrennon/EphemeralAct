use crate::application::dtos::{MergeRunExecutionsRequest, WorkflowExecution};

/// Inbound port for reducing a run's workflow executions to the one execution
/// the run reports.
pub trait MergeRunExecutionsPort {
    /// Merges the executions the run produced.
    fn execute(
        &self,
        request: MergeRunExecutionsRequest,
    ) -> Result<WorkflowExecution, Box<dyn std::error::Error>>;
}
