use std::{error::Error, sync::Arc};

use crate::application::commands::ExecuteJobCommand;
use crate::{
    application::{
        dtos::{ExecuteWorkflowRequest, JobSummary, LoadWorkflowRequest, WorkflowExecution},
        ports::{
            inbound::execute_workflow_port::ExecuteWorkflowPort,
            outbound::{command_bus_port::CommandBusPort, load_workflow_port::LoadWorkflowPort},
        },
    },
    domain::planner::Planner,
};

/// Application service coordinating the execution of a single workflow.
///
/// Loads the workflow definition through an outbound port, plans its job
/// stages, and publishes one [`ExecuteJobCommand`] per planned run. The job
/// command handler is what turns each command into an execution, so this
/// service never depends on the job entrypoint itself.
pub struct ExecuteWorkflowService {
    workflow_loader: Box<dyn LoadWorkflowPort>,
    command_bus: Arc<dyn CommandBusPort>,
}

impl ExecuteWorkflowService {
    pub fn new(
        workflow_loader: Box<dyn LoadWorkflowPort>,
        command_bus: Arc<dyn CommandBusPort>,
    ) -> Self {
        Self {
            workflow_loader,
            command_bus,
        }
    }
}

impl ExecuteWorkflowPort for ExecuteWorkflowService {
    fn execute(
        &self,
        request: ExecuteWorkflowRequest<'_>,
    ) -> Result<WorkflowExecution, Box<dyn Error>> {
        let workflow = self.workflow_loader.execute(LoadWorkflowRequest {
            workflow_content: request.workflow_content,
        })?;
        let workflow_name = workflow.name.clone().unwrap_or_else(|| "unnamed".into());
        let plan = Planner.plan(&workflow).map_err(|e| format!("{:?}", e))?;

        let mut job_summaries: Vec<JobSummary> = Vec::new();
        let mut container_names: Vec<String> = Vec::new();
        let mut success = true;

        for stage in &plan.stages {
            for run in &stage.runs {
                let execution = self.command_bus.dispatch_job(ExecuteJobCommand::new(
                    run.job.clone(),
                    run.job_id.clone(),
                    workflow.clone(),
                    request.repo_path.to_path_buf(),
                    request.context.clone(),
                ))?;
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
