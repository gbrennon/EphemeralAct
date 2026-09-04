use std::sync::Arc;

use crate::{
    application::{
        ports::outbound::{EventBusPort, WorkflowSourcePort},
        services::{
            list_actions_service::ListActionsService, list_workflows_service::ListWorkflowsService,
            run_action_service::RunActionService,
            run_all_workflows_service::RunAllWorkflowsService,
            run_workflow_service::RunWorkflowService,
        },
    },
    infrastructure::{
        actions::{ActionFetcherPort, GitActionFetcher},
        containers::{ContainerCleanupHandler, ContainerRuntimeAdapter, ContainerRuntimePort},
        di::{app_container::AppContainer, command_bus_wiring::CommandBusWiring},
        images::{ImageMapperPort, PlatformImageMapper},
        messaging::InMemoryEventBus,
        workflows::FilesystemWorkflowSource,
    },
};

pub struct Container;

impl Container {
    pub fn build() -> AppContainer {
        let runtime: Arc<dyn ContainerRuntimePort> = Arc::new(
            ContainerRuntimeAdapter::detect()
                .expect("no container runtime available (Docker or Podman required)"),
        );
        Self::with_runtime(runtime)
    }

    pub fn with_runtime(runtime: Arc<dyn ContainerRuntimePort>) -> AppContainer {
        Self::with_collaborators(
            runtime,
            Box::new(PlatformImageMapper),
            Box::new(GitActionFetcher::with_default_cache_root()),
            Arc::new(FilesystemWorkflowSource::default()),
        )
    }

    pub fn with_collaborators(
        runtime: Arc<dyn ContainerRuntimePort>,
        image_mapper: Box<dyn ImageMapperPort>,
        action_fetcher: Box<dyn ActionFetcherPort>,
        workflow_source: Arc<dyn WorkflowSourcePort>,
    ) -> AppContainer {
        let event_bus: Arc<dyn EventBusPort> = Arc::new(InMemoryEventBus::new(Box::new(
            ContainerCleanupHandler::new(runtime.clone()),
        )));
        let command_bus = CommandBusWiring::build(runtime, image_mapper, action_fetcher);
        let list_workflows_service = ListWorkflowsService::new(Box::new(workflow_source.clone()));
        let list_actions_service = ListActionsService::new(Box::new(workflow_source.clone()));
        let run_workflow_service = RunWorkflowService::new(
            Box::new(workflow_source.clone()),
            command_bus.clone(),
            event_bus.clone(),
        );
        let run_all_workflows_service =
            RunAllWorkflowsService::new(Box::new(workflow_source), command_bus.clone(), event_bus);
        let run_action_service = RunActionService::new(command_bus);

        AppContainer {
            run_all_workflows_port: Box::new(run_all_workflows_service),
            run_workflow_port: Box::new(run_workflow_service),
            run_action_port: Box::new(run_action_service),
            list_workflows_port: Box::new(list_workflows_service),
            list_actions_port: Box::new(list_actions_service),
        }
    }
}
