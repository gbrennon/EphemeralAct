use crate::{
    application::{
        dtos::{
            ExecuteActionRequest, ExecuteActionResponse, LoadActionDefinitionRequest,
            ResolveActionDirectoryRequest, ResolveActionInputsRequest, ResolvedActionDirectory,
            RunCompositeActionRequest, RunNodeActionRequest,
        },
        ports::inbound::{
            execute_action_port::ExecuteActionPort,
            execute_nested_action_port::ExecuteNestedActionPort,
            load_action_definition_port::LoadActionDefinitionPort,
            resolve_action_directory_port::ResolveActionDirectoryPort,
            resolve_action_inputs_port::ResolveActionInputsPort,
            run_composite_action_port::RunCompositeActionPort,
            run_node_action_port::RunNodeActionPort,
        },
    },
    domain::{
        errors::{ActionError, StepError},
        workflow::ActionRuns,
    },
};

/// Application service that runs the action a step references.
///
/// Resolves where the action lives, loads its definition and inputs, and
/// dispatches on how the action declares it runs: composite actions run their
/// steps in the job's container, JavaScript actions are copied in and run with
/// node, and container actions are reported as unsupported instead of being
/// silently skipped.
pub struct ExecuteActionService {
    directory_resolver: Box<dyn ResolveActionDirectoryPort>,
    definition_loader: Box<dyn LoadActionDefinitionPort>,
    input_resolver: Box<dyn ResolveActionInputsPort>,
    composite_runner: Box<dyn RunCompositeActionPort>,
    node_runner: Box<dyn RunNodeActionPort>,
}

impl ExecuteActionService {
    pub fn new(
        directory_resolver: Box<dyn ResolveActionDirectoryPort>,
        definition_loader: Box<dyn LoadActionDefinitionPort>,
        input_resolver: Box<dyn ResolveActionInputsPort>,
        composite_runner: Box<dyn RunCompositeActionPort>,
        node_runner: Box<dyn RunNodeActionPort>,
    ) -> Self {
        Self {
            directory_resolver,
            definition_loader,
            input_resolver,
            composite_runner,
            node_runner,
        }
    }

    fn run_action(
        &self,
        request: &ExecuteActionRequest,
    ) -> Result<ExecuteActionResponse, StepError> {
        let action_dir = match self
            .directory_resolver
            .execute(ResolveActionDirectoryRequest {
                action_ref: &request.action_ref,
                repo_path: &request.repo_path,
            })? {
            ResolvedActionDirectory::Skipped(response) => return Ok(response),
            ResolvedActionDirectory::Directory(directory) => directory,
        };

        let definition = self
            .definition_loader
            .execute(LoadActionDefinitionRequest {
                action_dir: &action_dir,
            })
            .map_err(|error| {
                StepError::new(format!(
                    "failed to load action '{}': {}",
                    request.action_ref, error.message
                ))
            })?;
        let inputs = self.input_resolver.execute(ResolveActionInputsRequest {
            definition: &definition,
            step: &request.step,
        });

        match &definition.runs {
            ActionRuns::Composite { steps } => {
                self.composite_runner.execute(RunCompositeActionRequest {
                    steps,
                    inputs: &inputs,
                    action_dir: &action_dir,
                    action_request: request,
                    nested_executor: self,
                })
            }
            ActionRuns::Node12 { main }
            | ActionRuns::Node16 { main }
            | ActionRuns::Node20 { main } => self
                .node_runner
                .execute(RunNodeActionRequest {
                    action_dir: &action_dir,
                    entry_point: main,
                    inputs: &inputs,
                    env: &request.env,
                    container: request.container.as_ref(),
                })
                .map(|result| ExecuteActionResponse {
                    exit_code: result.exit_code,
                    stdout: result.stdout,
                    stderr: result.stderr,
                }),
            ActionRuns::Docker { image } => Err(StepError::new(
                ActionError::Unsupported(format!(
                    "action '{}' runs the container image '{image}', which cannot be executed yet",
                    request.action_ref
                ))
                .to_string(),
            )),
        }
    }
}

impl ExecuteActionPort for ExecuteActionService {
    fn execute(&self, request: ExecuteActionRequest) -> Result<ExecuteActionResponse, StepError> {
        self.run_action(&request)
    }
}

impl ExecuteNestedActionPort for ExecuteActionService {
    fn execute(&self, request: ExecuteActionRequest) -> Result<ExecuteActionResponse, StepError> {
        self.run_action(&request)
    }
}
