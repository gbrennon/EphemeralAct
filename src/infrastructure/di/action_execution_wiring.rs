use crate::application::{
    ports::outbound::ActionFetcherPort,
    services::{
        build_action_input_environment_service::BuildActionInputEnvironmentService,
        collect_action_files_service::CollectActionFilesService,
        copy_action_to_container_service::CopyActionToContainerService,
        execute_action_service::ExecuteActionService,
        fetch_remote_action_service::FetchRemoteActionService,
        load_action_definition_service::LoadActionDefinitionService,
        resolve_action_directory_service::ResolveActionDirectoryService,
        resolve_action_inputs_service::ResolveActionInputsService,
        resolve_node_binary_service::ResolveNodeBinaryService,
        run_composite_action_service::RunCompositeActionService,
        run_composite_step_service::RunCompositeStepService,
        run_node_action_service::RunNodeActionService, run_shell_step_service::RunShellStepService,
    },
};

/// Assembles the service graph behind [`ExecuteActionService`].
///
/// Every construction site - production and test - goes through this builder,
/// so the graph is described in exactly one place.
pub struct ActionExecutionWiring;

impl ActionExecutionWiring {
    pub fn build(fetcher: Box<dyn ActionFetcherPort>) -> ExecuteActionService {
        ExecuteActionService::new(
            Box::new(ResolveActionDirectoryService::new(Box::new(
                FetchRemoteActionService::new(fetcher),
            ))),
            Box::new(LoadActionDefinitionService::new()),
            Box::new(ResolveActionInputsService::new()),
            Box::new(RunCompositeActionService::new(Box::new(
                RunCompositeStepService::new(Box::new(RunShellStepService::new())),
            ))),
            Box::new(RunNodeActionService::new(
                Box::new(CopyActionToContainerService::new(Box::new(
                    CollectActionFilesService::new(),
                ))),
                Box::new(BuildActionInputEnvironmentService::new()),
                Box::new(ResolveNodeBinaryService::new()),
            )),
        )
    }
}
