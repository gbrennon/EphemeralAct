use std::error::Error;

use crate::application::{
    dtos::{MergeRunExecutionsRequest, WorkflowExecution},
    ports::outbound::merge_run_executions_port::MergeRunExecutionsPort,
};

/// Summary name used when every workflow in the repository is executed.
pub const ALL_WORKFLOWS_SUMMARY_NAME: &str = "all-workflows";

/// Service that reduces a run's workflow executions to the single execution the
/// run reports.
///
/// A run of one workflow reports that workflow unchanged; a run of every
/// workflow is reported as one execution whose job names name the workflow they
/// came from.
pub struct MergeRunExecutionsService;

impl MergeRunExecutionsService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MergeRunExecutionsService {
    fn default() -> Self {
        Self::new()
    }
}

impl MergeRunExecutionsPort for MergeRunExecutionsService {
    fn execute(
        &self,
        request: MergeRunExecutionsRequest,
    ) -> Result<WorkflowExecution, Box<dyn Error>> {
        if !request.all_workflows {
            return request
                .executions
                .into_iter()
                .next()
                .ok_or_else(|| "no workflow file resolved".into());
        }

        let mut merged = WorkflowExecution {
            workflow_name: ALL_WORKFLOWS_SUMMARY_NAME.into(),
            job_summaries: Vec::new(),
            container_names: Vec::new(),
            success: true,
        };

        for execution in request.executions {
            let WorkflowExecution {
                workflow_name,
                job_summaries,
                container_names,
                success,
            } = execution;

            merged
                .job_summaries
                .extend(job_summaries.into_iter().map(|mut job| {
                    job.name = job.name.map(|name| format!("{} / {}", workflow_name, name));
                    job
                }));
            merged.container_names.extend(container_names);
            merged.success &= success;
        }

        Ok(merged)
    }
}
