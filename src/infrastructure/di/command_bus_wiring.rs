use std::sync::Arc;

use crate::{
    application::{
        ports::outbound::CommandBusPort,
        services::{
            execute_job_service::ExecuteJobService, execute_step_service::ExecuteStepService,
            execute_workflow_service::ExecuteWorkflowService,
        },
    },
    infrastructure::{
        actions::{ActionCommandHandler, ActionFetcherPort},
        containers::{
            ContainerRuntimePort, create_job_container_service::CreateJobContainerService,
            prepare_job_container_service::PrepareJobContainerService,
            pull_job_image_service::PullJobImageService,
        },
        di::action_execution_wiring::ActionExecutionWiring,
        images::ImageMapperPort,
        jobs::{GitHubJobEnvironmentAdapter, JobCommandHandler},
        messaging::{DeferredCommandBus, InMemoryCommandBus},
        steps::{
            StepCommandHandler, build_step_context_service::BuildStepContextService,
            prefix_step_path_service::PrefixStepPathService,
            read_step_env_exports_service::ReadStepEnvExportsService,
            read_step_exports_service::ReadStepExportsService,
            read_step_path_exports_service::ReadStepPathExportsService,
            run_shell_step_service::RunShellStepService,
            summarize_step_service::SummarizeStepService,
        },
        workflows::{WorkflowCommandHandler, load_workflow_service::LoadWorkflowService},
    },
};

/// Assembles the command bus and the handler graph behind it.
///
/// Every coordination step of a run is a command: the workflow service
/// publishes job commands, the job service publishes step commands, and the
/// step service publishes action commands. Handlers are the only components
/// that know both a command and the application service handling it, so the
/// services are handed a [`DeferredCommandBus`] and the assembled bus is bound
/// into it once the graph is complete.
pub struct CommandBusWiring;

impl CommandBusWiring {
    #[must_use]
    pub fn build(
        runtime: Arc<dyn ContainerRuntimePort>,
        image_mapper: Box<dyn ImageMapperPort>,
        action_fetcher: Box<dyn ActionFetcherPort>,
    ) -> Arc<dyn CommandBusPort> {
        let image_mapper: Arc<dyn ImageMapperPort> = Arc::from(image_mapper);
        let deferred = Arc::new(DeferredCommandBus::new());
        let command_bus: Arc<dyn CommandBusPort> = deferred.clone();

        let workflow_handler = WorkflowCommandHandler::new(Box::new(ExecuteWorkflowService::new(
            Box::new(LoadWorkflowService::new()),
            command_bus.clone(),
        )));

        let job_handler = JobCommandHandler::new(Box::new(Self::build_job_executor(
            runtime.clone(),
            image_mapper,
            command_bus.clone(),
        )));

        let step_handler = StepCommandHandler::new(Box::new(ExecuteStepService::new(
            Box::new(RunShellStepService::new()),
            command_bus.clone(),
        )));

        let action_handler = ActionCommandHandler::new(Box::new(ActionExecutionWiring::build(
            action_fetcher,
            command_bus,
        )));

        deferred.bind(Box::new(InMemoryCommandBus::new(
            Box::new(workflow_handler),
            Box::new(job_handler),
            Box::new(step_handler),
            Box::new(action_handler),
        )));

        deferred
    }

    fn build_job_executor(
        runtime: Arc<dyn ContainerRuntimePort>,
        image_mapper: Arc<dyn ImageMapperPort>,
        command_bus: Arc<dyn CommandBusPort>,
    ) -> ExecuteJobService {
        ExecuteJobService::new(
            Box::new(GitHubJobEnvironmentAdapter::new()),
            Box::new(PrepareJobContainerService::new(
                Box::new(PullJobImageService::new(runtime.clone(), image_mapper)),
                Box::new(CreateJobContainerService::new(runtime)),
            )),
            Box::new(PrefixStepPathService::new()),
            Box::new(BuildStepContextService::new()),
            Box::new(SummarizeStepService::new()),
            Box::new(ReadStepExportsService::new(
                Box::new(ReadStepPathExportsService::new()),
                Box::new(ReadStepEnvExportsService::new()),
            )),
            command_bus,
        )
    }
}
