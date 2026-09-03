use std::{
    collections::HashMap,
    fs::{read, read_dir, read_to_string},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::{
    application::{
        dtos::{ExecuteActionRequest, ExecuteActionResponse},
        ports::{
            inbound::execute_action_port::ExecuteActionPort,
            outbound::{ActionFetcherPort, ContainerPort, ExecResult, FileEntry},
        },
    },
    domain::{
        errors::{ActionError, StepError},
        expression::{EvalContext, StepInterpolator},
        value_objects::{ActionReference, ShellCommand},
        workflow::{ActionDefinition, ActionRuns, Step},
    },
};

/// Repository name whose action only checks out the repository, which the
/// runner already provides by mounting the workspace.
const CHECKOUT_REPO: &str = "checkout";

/// Directory inside the container that holds actions copied in for a run.
const CONTAINER_ACTIONS_ROOT: &str = "/tmp/ephemeral-act-actions";

/// Directory never copied into the container along with an action.
const GIT_DIRECTORY: &str = ".git";

/// Working directory of an action's entry point inside the container.
const CONTAINER_WORKSPACE: &str = "/workspace";

/// Interpreter used for JavaScript actions when the container exposes no
/// absolute path for it.
const NODE_COMMAND: &str = "node";

/// Application service that resolves and runs the action a step references.
///
/// Local references (`./path`) are read from the repository under test; remote
/// references are retrieved through the [`ActionFetcherPort`], so an action
/// hosted on GitHub, Forgejo, or any other forge resolves the same way.
/// Composite actions run their steps in the job's container, JavaScript actions
/// are copied into the container and run with node, and container actions are
/// reported as unsupported instead of being silently skipped.
pub struct ExecuteActionService<F: ActionFetcherPort> {
    fetcher: F,
}

impl<F: ActionFetcherPort> ExecuteActionService<F> {
    pub fn new(fetcher: F) -> Self {
        Self { fetcher }
    }

    fn run_action(
        &self,
        request: &ExecuteActionRequest,
    ) -> Result<ExecuteActionResponse, StepError> {
        let reference =
            ActionReference::parse(&request.action_ref).map_err(Self::action_error_to_step)?;

        let action_dir = match &reference {
            ActionReference::Local(path) => request.repo_path.join(path.trim_start_matches("./")),
            ActionReference::Docker(image) => {
                return Err(Self::action_error_to_step(ActionError::Unsupported(
                    format!("container action '{image}' cannot be executed yet"),
                )));
            }
            ActionReference::Remote(remote) if remote.repo() == CHECKOUT_REPO => {
                return Ok(ExecuteActionResponse::note(format!(
                    "[skipped] {} - the repository is already mounted at {CONTAINER_WORKSPACE}\n",
                    request.action_ref
                )));
            }
            ActionReference::Remote(remote) => {
                let fetched = self
                    .fetcher
                    .fetch(remote)
                    .map_err(Self::action_error_to_step)?;
                match remote.directory() {
                    Some(directory) => fetched.join(directory),
                    None => fetched,
                }
            }
        };

        let definition = Self::load_definition(&action_dir).map_err(|error| {
            StepError::new(format!(
                "failed to load action '{}': {error}",
                request.action_ref
            ))
        })?;
        let inputs = Self::resolve_inputs(&definition, &request.step);

        match &definition.runs {
            ActionRuns::Composite { steps } => {
                self.run_composite(steps, &inputs, request, &action_dir)
            }
            ActionRuns::Node16 { main } | ActionRuns::Node20 { main } => {
                Self::run_node(&action_dir, main, &inputs, request).map(Self::to_response)
            }
            ActionRuns::Docker { image } => Err(Self::action_error_to_step(
                ActionError::Unsupported(format!(
                    "action '{}' runs the container image '{image}', which cannot be executed yet",
                    request.action_ref
                )),
            )),
        }
    }

    fn run_composite(
        &self,
        steps: &[Step],
        inputs: &HashMap<String, String>,
        request: &ExecuteActionRequest,
        action_dir: &Path,
    ) -> Result<ExecuteActionResponse, StepError> {
        let context = Self::context_with_inputs(&request.context, inputs);
        let mut stdout = String::new();
        let mut stderr = String::new();

        for step in steps {
            let interpolated =
                StepInterpolator::interpolate(step, &context).map_err(|error| StepError {
                    message: format!("failed to resolve expressions: {error:?}"),
                    stdout: stdout.clone(),
                    stderr: stderr.clone(),
                })?;

            let outcome = match interpolated.uses() {
                Some(nested) => self
                    .run_action(&ExecuteActionRequest {
                        action_ref: nested.to_string(),
                        step: interpolated.clone(),
                        repo_path: request.repo_path.clone(),
                        env: request.env.clone(),
                        context: context.clone(),
                        container: request.container.clone(),
                    })
                    .map(|response| ExecResult {
                        exit_code: response.exit_code,
                        stdout: response.stdout,
                        stderr: response.stderr,
                    }),
                None => Self::run_shell_step(
                    &interpolated,
                    request.container.as_ref(),
                    &request.env,
                    action_dir,
                ),
            };

            match outcome {
                Ok(result) => {
                    stdout.push_str(&result.stdout);
                    stderr.push_str(&result.stderr);
                    if result.exit_code != 0 {
                        return Ok(ExecuteActionResponse {
                            exit_code: result.exit_code,
                            stdout,
                            stderr,
                        });
                    }
                }
                Err(error) => {
                    return Err(StepError {
                        message: error.message,
                        stdout: format!("{stdout}{}", error.stdout),
                        stderr: format!("{stderr}{}", error.stderr),
                    });
                }
            }
        }

        Ok(ExecuteActionResponse {
            exit_code: 0,
            stdout,
            stderr,
        })
    }

    fn run_shell_step(
        step: &Step,
        container: &dyn ContainerPort,
        env: &HashMap<String, String>,
        action_dir: &Path,
    ) -> Result<ExecResult, StepError> {
        let mut action_env = env.clone();
        action_env.insert(
            "GITHUB_ACTION_PATH".into(),
            action_dir.display().to_string(),
        );

        let command = ShellCommand::for_step(step, &action_env)
            .ok_or_else(|| StepError::new("step has neither `run` nor `uses` defined"))?;

        container
            .exec(command.argv(), command.working_directory(), command.env())
            .map_err(|error| StepError::new(format!("{error:?}")))
    }

    /// Copies a JavaScript action into the container and runs its entry point,
    /// exposing inputs as `INPUT_<NAME>` variables the way a real runner does.
    fn run_node(
        action_dir: &Path,
        entry_point: &str,
        inputs: &HashMap<String, String>,
        request: &ExecuteActionRequest,
    ) -> Result<ExecResult, StepError> {
        let container = request.container.as_ref();
        let container_dir = Self::container_action_dir(action_dir);
        let files = Self::collect_action_files(action_dir)?;

        container
            .exec(
                &["mkdir".into(), "-p".into(), container_dir.clone()],
                None,
                &HashMap::new(),
            )
            .map_err(|error| {
                StepError::new(format!("failed to create action directory: {error:?}"))
            })?;
        container
            .copy_to(&container_dir, &files)
            .map_err(|error| StepError::new(format!("failed to copy action files: {error:?}")))?;

        let mut action_env = request.env.clone();
        action_env.insert("GITHUB_ACTION_PATH".into(), container_dir.clone());
        for (name, value) in inputs {
            action_env.insert(Self::input_variable(name), value.clone());
        }

        let command = ShellCommand::new(
            vec![
                Self::resolve_node_binary(container),
                format!("{container_dir}/{entry_point}"),
            ],
            Some(CONTAINER_WORKSPACE.into()),
            action_env,
        );

        container
            .exec(command.argv(), command.working_directory(), command.env())
            .map_err(|error| StepError::new(format!("failed to run node action: {error:?}")))
    }

    /// Returns the node interpreter to run a JavaScript action with.
    ///
    /// Runner images commonly install node in a tool cache that only a login
    /// shell puts on `PATH`, so the binary is looked up through one; when the
    /// lookup finds nothing, the bare command is used so the failure names the
    /// missing interpreter.
    fn resolve_node_binary(container: &dyn ContainerPort) -> String {
        container
            .exec(
                &["bash".into(), "-lc".into(), "command -v node".into()],
                None,
                &HashMap::new(),
            )
            .ok()
            .filter(|result| result.exit_code == 0)
            .map(|result| result.stdout.trim().to_string())
            .filter(|path| !path.is_empty())
            .unwrap_or_else(|| NODE_COMMAND.to_string())
    }

    fn resolve_inputs(definition: &ActionDefinition, step: &Step) -> HashMap<String, String> {
        let mut inputs: HashMap<String, String> = definition
            .inputs
            .iter()
            .filter_map(|(name, input)| {
                input
                    .default
                    .as_ref()
                    .map(|default| (name.clone(), default.clone()))
            })
            .collect();
        inputs.extend(
            step.with
                .iter()
                .map(|(name, value)| (name.clone(), value.clone())),
        );
        inputs
    }

    fn context_with_inputs(context: &EvalContext, inputs: &HashMap<String, String>) -> EvalContext {
        let mut action_context = context.clone();
        action_context.inputs = Value::Object(
            inputs
                .iter()
                .map(|(name, value)| (name.clone(), Value::String(value.clone())))
                .collect(),
        );
        action_context
    }

    fn load_definition(action_dir: &Path) -> Result<ActionDefinition, String> {
        let candidates = [
            action_dir.join("action.yml"),
            action_dir.join("action.yaml"),
        ];
        let path = candidates
            .iter()
            .find(|candidate| candidate.exists())
            .ok_or_else(|| format!("action.yml not found in {}", action_dir.display()))?;

        let contents = read_to_string(path)
            .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
        serde_yaml::from_str(&contents)
            .map_err(|error| format!("failed to parse {}: {error}", path.display()))
    }

    fn container_action_dir(action_dir: &Path) -> String {
        let slug: String = action_dir
            .display()
            .to_string()
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() || character == '.' || character == '-' {
                    character
                } else {
                    '_'
                }
            })
            .collect();
        format!("{CONTAINER_ACTIONS_ROOT}/{slug}")
    }

    fn input_variable(name: &str) -> String {
        format!("INPUT_{}", name.to_uppercase().replace(' ', "_"))
    }

    fn collect_action_files(action_dir: &Path) -> Result<Vec<FileEntry>, StepError> {
        let mut files = Vec::new();
        Self::collect_files_into(action_dir, action_dir, &mut files)?;
        Ok(files)
    }

    fn collect_files_into(
        root: &Path,
        directory: &Path,
        files: &mut Vec<FileEntry>,
    ) -> Result<(), StepError> {
        let listing = read_dir(directory).map_err(|error| {
            StepError::new(format!(
                "failed to read action directory {}: {error}",
                directory.display()
            ))
        })?;

        for entry in listing {
            let path: PathBuf = entry
                .map_err(|error| StepError::new(format!("failed to read action entry: {error}")))?
                .path();

            if path.file_name().is_some_and(|name| name == GIT_DIRECTORY) {
                continue;
            }
            if path.is_dir() {
                Self::collect_files_into(root, &path, files)?;
                continue;
            }

            let relative = path
                .strip_prefix(root)
                .map_err(|error| StepError::new(format!("action file outside action: {error}")))?;
            let content = read(&path).map_err(|error| {
                StepError::new(format!("failed to read {}: {error}", path.display()))
            })?;
            let mode = path
                .metadata()
                .map(|metadata| metadata.permissions().mode() & 0o7777)
                .unwrap_or(0o644);

            files.push(FileEntry {
                path: relative.display().to_string(),
                content,
                mode,
            });
        }

        Ok(())
    }

    fn to_response(result: ExecResult) -> ExecuteActionResponse {
        ExecuteActionResponse {
            exit_code: result.exit_code,
            stdout: result.stdout,
            stderr: result.stderr,
        }
    }

    fn action_error_to_step(error: ActionError) -> StepError {
        StepError::new(error.to_string())
    }
}

impl<F: ActionFetcherPort> ExecuteActionPort for ExecuteActionService<F> {
    fn execute(&self, request: ExecuteActionRequest) -> Result<ExecuteActionResponse, StepError> {
        self.run_action(&request)
    }
}
