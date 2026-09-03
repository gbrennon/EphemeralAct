use std::error::Error;

use crate::{
    application::{
        dtos::{
            ExecuteJobRequest, ExecuteWorkflowRequest, JobSummary, LoadWorkflowRequest,
            WorkflowExecution,
        },
        ports::inbound::{
            execute_job_port::ExecuteJobPort, execute_workflow_port::ExecuteWorkflowPort,
            load_workflow_port::LoadWorkflowPort,
        },
    },
    domain::planner::Planner,
};

/// Service that runs every job of one workflow file in the order the planner
/// derives from their dependencies.
pub struct ExecuteWorkflowService {
    workflow_loader: Box<dyn LoadWorkflowPort>,
    job_executor: Box<dyn ExecuteJobPort>,
}

impl ExecuteWorkflowService {
    pub fn new(
        workflow_loader: Box<dyn LoadWorkflowPort>,
        job_executor: Box<dyn ExecuteJobPort>,
    ) -> Self {
        Self {
            workflow_loader,
            job_executor,
        }
    }
}

impl ExecuteWorkflowPort for ExecuteWorkflowService {
    fn execute(
        &self,
        request: ExecuteWorkflowRequest<'_>,
    ) -> Result<WorkflowExecution, Box<dyn Error>> {
        let workflow = self.workflow_loader.execute(LoadWorkflowRequest {
            workflow_file: request.workflow_file,
        })?;
        let workflow_name = workflow.name.clone().unwrap_or_else(|| "unnamed".into());
        let plan = Planner.plan(&workflow).map_err(|e| format!("{:?}", e))?;

        let mut job_summaries: Vec<JobSummary> = Vec::new();
        let mut container_names: Vec<String> = Vec::new();
        let mut success = true;

        for stage in &plan.stages {
            for run in &stage.runs {
                let execution = self.job_executor.execute(ExecuteJobRequest {
                    run,
                    workflow: &workflow,
                    repo_path: request.repo_path,
                    context: request.context,
                })?;
                success &= execution.job_summary.success;
                job_summaries.push(execution.job_summary);
                container_names.push(execution.container_name);
            }
        }

        Ok(WorkflowExecution {
            workflow_name,
            job_summaries,
            container_names,
            success,
        })
    }
}
