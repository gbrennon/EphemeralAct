use crate::application::ports::inbound::execute_job_port::ExecuteJobPort;
use std::error::Error;

use crate::application::commands::ExecuteJobCommand;
use crate::{
    application::dtos::{ExecuteJobRequest, JobExecution},
    domain::planner::Run,
};

/// Infrastructure command handler that processes `ExecuteJobCommand`.
pub struct JobCommandHandler {
    executor: Box<dyn ExecuteJobPort>,
}

impl JobCommandHandler {
    pub fn new(executor: Box<dyn ExecuteJobPort>) -> Self {
        Self { executor }
    }

    pub fn handle(&self, cmd: ExecuteJobCommand) -> Result<JobExecution, Box<dyn Error>> {
        let run = Run {
            workflow_name: cmd.workflow.name.clone(),
            job_id: cmd.job_id,
            job: cmd.job,
            matrix_values: None,
        };

        let req = ExecuteJobRequest {
            run: &run,
            workflow: &cmd.workflow,
            repo_path: &cmd.repo_path,
            context: &cmd.context,
        };
        self.executor.execute(req)
    }
}
