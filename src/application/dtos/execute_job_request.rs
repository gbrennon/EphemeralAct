use std::path::Path;

use crate::domain::{expression::EvalContext, planner::Run, workflow::Workflow};

/// Request DTO for the
/// [`ExecuteJobPort`](crate::application::ports::inbound::execute_job_port::ExecuteJobPort)
/// inbound port.
pub struct ExecuteJobRequest<'a> {
    /// Planned job to run.
    pub run: &'a Run,
    /// Workflow the job belongs to.
    pub workflow: &'a Workflow,
    /// Repository directory the run executes against.
    pub repo_path: &'a Path,
    /// Context the job's steps are evaluated against.
    pub context: &'a EvalContext,
}
