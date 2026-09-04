use std::{error::Error, sync::Arc, time::Instant};

use crate::application::commands::ExecuteWorkflowCommand;
use crate::{
    application::{
        dtos::{RunAllWorkflowsRequest, RunSummary},
        ports::{
            inbound::run_all_workflows_port::RunAllWorkflowsPort,
            outbound::{CommandBusPort, EventBusPort, WorkflowSourcePort},
        },
    },
    domain::events::{ActRunCompletedPayload, DomainEvent},
};

/// Name reported for the aggregate summary of a full multi-workflow run.
pub const ALL_WORKFLOWS_SUMMARY_NAME: &str = "All Workflows";

/// Application service implementing the entrypoint to run all workflows.
///
/// Agnostic by construction: it never touches the filesystem. It reads workflow
/// definitions through the outbound [`WorkflowSourcePort`] and dispatches commands
/// through the outbound [`CommandBusPort`].
pub struct RunAllWorkflowsService {
    workflow_source: Box<dyn WorkflowSourcePort>,
    command_bus: Arc<dyn CommandBusPort>,
    event_bus: Arc<dyn EventBusPort>,
}

impl RunAllWorkflowsService {
    pub fn new(
        workflow_source: Box<dyn WorkflowSourcePort>,
        command_bus: Arc<dyn CommandBusPort>,
        event_bus: Arc<dyn EventBusPort>,
    ) -> Self {
        Self {
            workflow_source,
            command_bus,
            event_bus,
        }
    }
}

impl RunAllWorkflowsPort for RunAllWorkflowsService {
    fn execute(&self, request: RunAllWorkflowsRequest) -> Result<RunSummary, Box<dyn Error>> {
        let started_at = Instant::now();
        let repo = &request.repository;

        // Use the outbound port to get all workflow contents
        let workflow_contents = self.workflow_source.read_all_workflows(repo)?;

        let mut executions = Vec::new();
        for workflow_content in workflow_contents {
            let execution = self
                .command_bus
                .dispatch_workflow(ExecuteWorkflowCommand::new(
                    workflow_content,
                    request.config.clone(),
                    repo.clone(),
                ))?;
            executions.push(execution);
        }

        let success = executions.iter().all(|e| e.success);

        let mut all_jobs = Vec::new();
        for exec in &executions {
            for job in &exec.job_summaries {
                let mut j = job.clone();
                if let Some(name) = &j.name {
                    j.name = Some(format!("{} / {}", exec.workflow_name, name));
                }
                all_jobs.push(j);
            }
        }

        let container_names: Vec<String> = executions
            .iter()
            .flat_map(|e| e.container_names.clone())
            .collect();

        self.event_bus
            .publish(DomainEvent::ActRunCompleted(ActRunCompletedPayload {
                container_names,
                success,
            }));

        Ok(RunSummary {
            name: ALL_WORKFLOWS_SUMMARY_NAME.into(),
            success,
            duration: started_at.elapsed(),
            job_summaries: all_jobs,
        })
    }
}
