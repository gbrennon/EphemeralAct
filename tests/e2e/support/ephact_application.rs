use ephact::{
    application::{
        ports::outbound::{ActionFetcherPort, ContainerRuntimePort},
        services::{
            container_cleanup_service::ContainerCleanupService,
            execute_action_service::ExecuteActionService, list_actions_service::ListActionsService,
            list_workflows_service::ListWorkflowsService, run_act_service::RunActService,
        },
    },
    infrastructure::{FilesystemWorkflowFileParser, InMemoryEventBus},
    presentation::composition_root::{Application, CompositionRoot},
};

use crate::fakes::fixed_image_mapper::FixedImageMapper;

/// Composes the application exactly as the production container does, with the
/// container runtime and the action fetcher replaced by test doubles so a
/// scenario never starts a container nor reaches a forge.
pub struct EphactApplication;

impl EphactApplication {
    pub fn compose<R, F>(runtime: R, fetcher: F) -> Application
    where
        R: ContainerRuntimePort + Clone + 'static,
        F: ActionFetcherPort + 'static,
    {
        let event_bus = InMemoryEventBus::new(
            Box::new(ContainerCleanupService::new(runtime.clone())),
            Box::new(ExecuteActionService::new(fetcher)),
        );

        CompositionRoot::compose(
            Box::new(RunActService::new(runtime, FixedImageMapper, event_bus)),
            Box::new(ListWorkflowsService::new(Box::new(
                FilesystemWorkflowFileParser,
            ))),
            Box::new(ListActionsService::new(Box::new(
                FilesystemWorkflowFileParser,
            ))),
        )
    }
}
