use std::sync::Arc;

use crate::{
    application::{
        ports::outbound::CommandBusPort, services::execute_action_service::ExecuteActionService,
    },
    infrastructure::{
        actions::{
            ActionFetcherPort, GitHubActionInputEnvironmentAdapter,
            collect_action_files_service::CollectActionFilesService,
            copy_action_to_container_service::CopyActionToContainerService,
            fetch_remote_action_service::FetchRemoteActionService,
            load_action_definition_service::LoadActionDefinitionService,
            resolve_action_directory_service::ResolveActionDirectoryService,
            resolve_action_inputs_service::ResolveActionInputsService,
            resolve_node_binary_service::ResolveNodeBinaryService,
            run_composite_action_service::RunCompositeActionService,
            run_node_action_service::RunNodeActionService,
        },
        steps::{
            run_composite_step_service::RunCompositeStepService,
            run_shell_step_service::RunShellStepService,
        },
    },
};

/// Assembles the action coordination service and the technical operations it
/// resolves and runs actions with.
///
/// The command bus is only needed for actions nested inside a composite
/// action: those are published as action commands instead of being called
/// directly.
pub struct ActionExecutionWiring;

impl ActionExecutionWiring {
    #[must_use]
    pub fn build(
        fetcher: Box<dyn ActionFetcherPort>,
        command_bus: Arc<dyn CommandBusPort>,
    ) -> ExecuteActionService {
        ExecuteActionService::new(
            Box::new(ResolveActionDirectoryService::new(Box::new(
                FetchRemoteActionService::new(fetcher),
            ))),
            Box::new(LoadActionDefinitionService::new()),
            Box::new(ResolveActionInputsService::new()),
            Box::new(RunCompositeActionService::new(Box::new(
                RunCompositeStepService::new(Box::new(RunShellStepService::new()), command_bus),
            ))),
            Box::new(RunNodeActionService::new(
                Box::new(CopyActionToContainerService::new(Box::new(
                    CollectActionFilesService::new(),
                ))),
                Box::new(GitHubActionInputEnvironmentAdapter::new()),
                Box::new(ResolveNodeBinaryService::new()),
            )),
        )
    }
}
