use std::sync::Arc;

use ephact::{
    application::ports::outbound::WorkflowSourcePort,
    infrastructure::{actions::ActionFetcherPort, containers::ContainerRuntimePort, di::Container},
    presentation::composition_root::{Application, CompositionRoot},
};

use crate::fakes::fixed_image_mapper::FixedImageMapper;

pub struct EphactApplication;

impl EphactApplication {
    pub fn compose(
        runtime: Arc<dyn ContainerRuntimePort>,
        fetcher: Box<dyn ActionFetcherPort>,
        workflow_source: Arc<dyn WorkflowSourcePort>,
    ) -> Application {
        let container = Container::with_collaborators(
            runtime,
            Box::new(FixedImageMapper),
            fetcher,
            workflow_source,
        );

        CompositionRoot::compose(
            container.run_workflow_port,
            container.run_all_workflows_port,
            container.list_workflows_port,
            container.list_actions_port,
        )
    }
}
