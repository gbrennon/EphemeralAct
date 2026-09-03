use std::sync::Arc;

use ephact::{
    application::{
        ports::outbound::{ActionFetcherPort, ContainerRuntimePort, EventPublisherPort},
        services::{
            container_cleanup_service::ContainerCleanupService,
            list_actions_service::ListActionsService, list_workflows_service::ListWorkflowsService,
        },
    },
    infrastructure::{
        ActionExecutionWiring, FilesystemWorkflowFileParser, InMemoryEventBus, RunActWiring,
    },
    presentation::composition_root::{Application, CompositionRoot},
};

use crate::fakes::fixed_image_mapper::FixedImageMapper;

/// Composes the application exactly as the production container does, with the
/// container runtime and the action fetcher replaced by test doubles so a
/// scenario never starts a container nor reaches a forge.
pub struct EphactApplication;

impl EphactApplication {
    pub fn compose(
        runtime: Arc<dyn ContainerRuntimePort>,
        fetcher: Box<dyn ActionFetcherPort>,
    ) -> Application {
        let event_bus: Arc<dyn EventPublisherPort> = Arc::new(InMemoryEventBus::new(
            Box::new(ContainerCleanupService::new(runtime.clone())),
            Box::new(ActionExecutionWiring::build(fetcher)),
        ));

        CompositionRoot::compose(
            Box::new(RunActWiring::build(
                runtime,
                Box::new(FixedImageMapper),
                event_bus,
            )),
            Box::new(ListWorkflowsService::new(Box::new(
                FilesystemWorkflowFileParser,
            ))),
            Box::new(ListActionsService::new(Box::new(
                FilesystemWorkflowFileParser,
            ))),
        )
    }
}
