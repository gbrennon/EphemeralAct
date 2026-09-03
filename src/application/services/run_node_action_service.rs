use crate::{
    application::{
        constants::CONTAINER_WORKSPACE,
        dtos::{
            BuildActionInputEnvironmentRequest, CopyActionToContainerRequest,
            ResolveNodeBinaryRequest, RunNodeActionRequest,
        },
        ports::outbound::{
            ExecResult, build_action_input_environment_port::BuildActionInputEnvironmentPort,
            copy_action_to_container_port::CopyActionToContainerPort,
            resolve_node_binary_port::ResolveNodeBinaryPort,
            run_node_action_port::RunNodeActionPort,
        },
    },
    domain::{errors::StepError, value_objects::ShellCommand},
};

/// Service that runs a JavaScript action: copies it into the container,
/// exposes its inputs as environment variables, and runs its entry point.
pub struct RunNodeActionService {
    action_copier: Box<dyn CopyActionToContainerPort>,
    environment_builder: Box<dyn BuildActionInputEnvironmentPort>,
    node_binary_resolver: Box<dyn ResolveNodeBinaryPort>,
}

impl RunNodeActionService {
    pub fn new(
        action_copier: Box<dyn CopyActionToContainerPort>,
        environment_builder: Box<dyn BuildActionInputEnvironmentPort>,
        node_binary_resolver: Box<dyn ResolveNodeBinaryPort>,
    ) -> Self {
        Self {
            action_copier,
            environment_builder,
            node_binary_resolver,
        }
    }
}

impl RunNodeActionPort for RunNodeActionService {
    fn execute(&self, request: RunNodeActionRequest<'_>) -> Result<ExecResult, StepError> {
        let container_dir = self.action_copier.execute(CopyActionToContainerRequest {
            action_dir: request.action_dir,
            container: request.container,
        })?;

        let action_env = self
            .environment_builder
            .execute(BuildActionInputEnvironmentRequest {
                env: request.env,
                inputs: request.inputs,
                action_path: &container_dir,
            });

        let binary = self.node_binary_resolver.execute(ResolveNodeBinaryRequest {
            container: request.container,
        });

        let entry_point = request.entry_point;
        let command = ShellCommand::new(
            vec![binary, format!("{container_dir}/{entry_point}")],
            Some(CONTAINER_WORKSPACE.into()),
            action_env,
        );

        request
            .container
            .exec(command.argv(), command.working_directory(), command.env())
            .map_err(|error| StepError::new(format!("failed to run node action: {error:?}")))
    }
}
