use std::sync::Arc;

use crate::{
    application::{
        ports::outbound::{ContainerRuntimePort, EventPublisherPort},
        services::{
            container_cleanup_service::ContainerCleanupService,
            list_actions_service::ListActionsService, list_workflows_service::ListWorkflowsService,
        },
    },
    infrastructure::{
        actions::GitActionFetcher,
        di::{
            action_execution_wiring::ActionExecutionWiring, app_container::AppContainer,
            run_act_wiring::RunActWiring,
        },
        events::InMemoryEventBus,
        images::PlatformImageMapper,
        runners::ContainerRuntimeAdapter,
        workflows::FilesystemWorkflowFileParser,
    },
};

/// Dependency-injection container that constructs and wires all application
/// dependencies. Returns a fully-wired [`AppContainer`] ready for the
/// presentation layer to consume.
pub struct Container;

impl Container {
    /// Builds the application service graph and returns all three ports.
    #[allow(clippy::arc_with_non_send_sync)]
    pub fn build() -> AppContainer {
        let runtime: Arc<dyn ContainerRuntimePort> = Arc::new(
            ContainerRuntimeAdapter::detect()
                .expect("no container runtime available (Docker or Podman required)"),
        );
        let event_bus: Arc<dyn EventPublisherPort> = Arc::new(InMemoryEventBus::new(
            Box::new(ContainerCleanupService::new(runtime.clone())),
            Box::new(ActionExecutionWiring::build(Box::new(
                GitActionFetcher::with_default_cache_root(),
            ))),
        ));

        let list_workflows_service =
            ListWorkflowsService::new(Box::new(FilesystemWorkflowFileParser));
        let list_actions_service = ListActionsService::new(Box::new(FilesystemWorkflowFileParser));
        let run_act_service =
            RunActWiring::build(runtime, Box::new(PlatformImageMapper), event_bus);

        AppContainer {
            run_act_port: Box::new(run_act_service),
            list_workflows_port: Box::new(list_workflows_service),
            list_actions_port: Box::new(list_actions_service),
        }
    }
}
