#![allow(dead_code)]
use std::{cell::RefCell, rc::Rc};

use ephact::application::{
    dtos::{ExecuteJobRequest, JobExecution, JobSummary},
    ports::inbound::execute_job_port::ExecuteJobPort,
};

/// Reports every job as run, recording the job ids in execution order and
/// failing the ones it was told to fail.
#[derive(Clone, Default)]
pub struct FakeExecuteJobPort {
    failing_job_ids: Vec<String>,
    executed_job_ids: Rc<RefCell<Vec<String>>>,
}

impl FakeExecuteJobPort {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn failing(failing_job_ids: Vec<String>) -> Self {
        Self {
            failing_job_ids,
            executed_job_ids: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn executed_job_ids(&self) -> Vec<String> {
        self.executed_job_ids.borrow().clone()
    }
}

impl ExecuteJobPort for FakeExecuteJobPort {
    fn execute(
        &self,
        request: ExecuteJobRequest<'_>,
    ) -> Result<JobExecution, Box<dyn std::error::Error>> {
        let job_id = request.run.job_id.clone();
        self.executed_job_ids.borrow_mut().push(job_id.clone());
        let success = !self.failing_job_ids.contains(&job_id);

        Ok(JobExecution {
            job_summary: JobSummary {
                job_id: job_id.clone(),
                name: request.run.job.name.clone(),
                steps: Vec::new(),
                success,
            },
            container_name: format!("container-{job_id}"),
        })
    }
}
