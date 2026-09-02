use std::sync::Arc;

use crate::{
    core::services::{
        container_cleanup_service::ContainerCleanupService,
        execute_action_service::ExecuteActionService, list_actions_service::ListActionsService,
        list_workflows_service::ListWorkflowsService, run_act_service::RunActService,
    },
    infrastructure::{
        actions::GitActionFetcher, di::app_container::AppContainer, events::InMemoryEventBus,
        images::PlatformImageMapper, runners::ContainerRuntimeAdapter,
        workflows::FilesystemWorkflowFileParser,
    },
};

/// Dependency-injection container that constructs and wires all application
/// dependencies. Returns a fully-wired [`AppContainer`] ready for the
/// presentation layer to consume.
pub struct Container;

impl Container {
    /// Builds the application service graph and returns all three ports.
    pub fn build() -> AppContainer {
        let runtime = Arc::new(
            ContainerRuntimeAdapter::detect()
                .expect("no container runtime available (Docker or Podman required)"),
        );
        let image_mapper = PlatformImageMapper;
        let cleanup_service = ContainerCleanupService::new(runtime.clone());
        let execute_action_service =
            ExecuteActionService::new(GitActionFetcher::with_default_cache_root());
        let event_bus =
            InMemoryEventBus::new(Box::new(cleanup_service), Box::new(execute_action_service));

        let parser = FilesystemWorkflowFileParser;
        let list_workflows_service = ListWorkflowsService::new(parser);
        let list_actions_service = ListActionsService::new(parser);
        let run_act_service = RunActService::new(runtime, image_mapper, event_bus);

        AppContainer {
            run_act_port: Box::new(run_act_service),
            list_workflows_port: Box::new(list_workflows_service),
            list_actions_port: Box::new(list_actions_service),
        }
    }
}
