use std::{error::Error, sync::Arc, time::Instant};

use crate::application::commands::ExecuteWorkflowCommand;
use crate::{
    application::{
        dtos::{RunSummary, RunWorkflowRequest},
        ports::{
            inbound::RunWorkflowPort,
            outbound::{CommandBusPort, EventBusPort, WorkflowSourcePort},
        },
    },
    domain::events::{ActRunCompletedPayload, DomainEvent},
};

/// Application service implementing the entrypoint to run a single workflow.
///
/// Agnostic by construction: it never touches files, containers, or any external
/// service. It reads the workflow definition through the outbound
/// [`WorkflowSourcePort`], expresses the intent to execute it as a command on the
/// outbound [`CommandBusPort`], and announces the outcome as a domain event on the
/// outbound [`EventBusPort`].
pub struct RunWorkflowService {
    workflow_source: Box<dyn WorkflowSourcePort>,
    command_bus: Arc<dyn CommandBusPort>,
    event_bus: Arc<dyn EventBusPort>,
}

impl RunWorkflowService {
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

impl RunWorkflowPort for RunWorkflowService {
    fn execute(&self, request: RunWorkflowRequest) -> Result<RunSummary, Box<dyn Error>> {
        let RunWorkflowRequest { config, repository } = request;
        let started_at = Instant::now();

        let workflow_content = self
            .workflow_source
            .read_workflow(&repository, config.workflow().map(|w| w.as_str()))?;

        let execution = self
            .command_bus
            .dispatch_workflow(ExecuteWorkflowCommand::new(
                workflow_content,
                config,
                repository,
            ))?;

        self.event_bus
            .publish(DomainEvent::ActRunCompleted(ActRunCompletedPayload {
                container_names: execution.container_names,
                success: execution.success,
            }));

        Ok(RunSummary {
            name: execution.workflow_name,
            success: execution.success,
            duration: started_at.elapsed(),
            job_summaries: execution.job_summaries,
        })
    }
}
