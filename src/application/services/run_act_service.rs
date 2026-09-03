use std::{error::Error, sync::Arc, time::Instant};

pub use crate::application::services::merge_run_executions_service::ALL_WORKFLOWS_SUMMARY_NAME;
use crate::{
    application::{
        dtos::{
            BuildRunContextRequest, ExecuteWorkflowRequest, MergeRunExecutionsRequest,
            ResolveWorkflowFilesRequest, RunActRequest, RunSummary, WorkflowExecution,
        },
        ports::{
            inbound::{
                build_run_context_port::BuildRunContextPort,
                execute_workflow_port::ExecuteWorkflowPort,
                merge_run_executions_port::MergeRunExecutionsPort,
                resolve_workflow_files_port::ResolveWorkflowFilesPort, run_act_port::RunActPort,
            },
            outbound::EventPublisherPort,
        },
    },
    domain::events::{ActRunCompletedPayload, DomainEvent},
};

/// Application service that executes a repository's workflows in containers.
///
/// Resolves which workflow files the run covers, builds the expression context
/// they are evaluated against, hands each file to the [`ExecuteWorkflowPort`],
/// and reports the merged outcome once every workflow has run.
pub struct RunActService {
    workflow_files_resolver: Box<dyn ResolveWorkflowFilesPort>,
    context_builder: Box<dyn BuildRunContextPort>,
    workflow_executor: Box<dyn ExecuteWorkflowPort>,
    execution_merger: Box<dyn MergeRunExecutionsPort>,
    event_publisher: Arc<dyn EventPublisherPort>,
}

impl RunActService {
    pub fn new(
        workflow_files_resolver: Box<dyn ResolveWorkflowFilesPort>,
        context_builder: Box<dyn BuildRunContextPort>,
        workflow_executor: Box<dyn ExecuteWorkflowPort>,
        execution_merger: Box<dyn MergeRunExecutionsPort>,
        event_publisher: Arc<dyn EventPublisherPort>,
    ) -> Self {
        Self {
            workflow_files_resolver,
            context_builder,
            workflow_executor,
            execution_merger,
            event_publisher,
        }
    }
}

impl RunActPort for RunActService {
    fn execute(&self, request: RunActRequest) -> Result<RunSummary, Box<dyn Error>> {
        let RunActRequest { config, repository } = request;
        let started_at = Instant::now();
        let repo_path = repository.path().as_path();

        let context = self
            .context_builder
            .execute(BuildRunContextRequest {
                config: &config,
                repository: &repository,
            })
            .context;

        let workflow_files = self
            .workflow_files_resolver
            .execute(ResolveWorkflowFilesRequest {
                config: &config,
                repo_path,
            })?
            .workflow_files;

        let executions = workflow_files
            .iter()
            .map(|workflow_file| {
                self.workflow_executor.execute(ExecuteWorkflowRequest {
                    workflow_file,
                    repo_path,
                    context: &context,
                })
            })
            .collect::<Result<Vec<WorkflowExecution>, _>>()?;

        let execution = self.execution_merger.execute(MergeRunExecutionsRequest {
            executions,
            all_workflows: config.all_workflows(),
        })?;

        self.event_publisher
            .publish(DomainEvent::ActRunCompleted(ActRunCompletedPayload {
                container_names: execution.container_names,
                success: execution.success,
            }));

        Ok(RunSummary {
            name: execution.workflow_name,
            job_summaries: execution.job_summaries,
            success: execution.success,
            duration: started_at.elapsed(),
        })
    }
}
