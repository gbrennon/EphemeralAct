use std::path::PathBuf;

use crate::domain::{
    expression::EvalContext,
    workflow::{Job, Workflow},
};

/// Command representing the intention to execute one job of a workflow.
///
/// Published by the workflow coordination service once the execution plan is
/// known, and handled by the job command handler.
#[derive(Debug, Clone)]
pub struct ExecuteJobCommand {
    pub job: Job,
    pub job_id: String,
    pub workflow: Workflow,
    pub repo_path: PathBuf,
    pub context: EvalContext,
}

impl ExecuteJobCommand {
    pub fn new(
        job: Job,
        job_id: String,
        workflow: Workflow,
        repo_path: PathBuf,
        context: EvalContext,
    ) -> Self {
        Self {
            job,
            job_id,
            workflow,
            repo_path,
            context,
        }
    }
}
