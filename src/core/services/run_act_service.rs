use std::{
    collections::HashMap,
    error::Error,
    fs::{read_dir, read_to_string},
    path::{Path, PathBuf},
    process,
    sync::Arc,
    time::Instant,
};

use serde_json::{Map, Value};

use super::workflow_execution::WorkflowExecution;
use crate::core::{
    ActRunConfig, Repository,
    dtos::{
        ExecuteActionRequest, ExecuteActionResponse, JobSummary, RunActRequest, RunSummary,
        StepSummary,
    },
    errors::StepError,
    events::{ActRunCompletedPayload, ActionExecutionRequestedPayload, DomainEvent, EventOutcome},
    expression::{EvalContext, StepInterpolator},
    planner::{Planner, Run},
    ports::{
        inbound::run_act_port::RunActPort,
        outbound::{
            ContainerConfig, ContainerPort, ContainerRuntimePort, EventPublisherPort,
            ImageMapperPort, RunnerContext,
        },
    },
    value_objects::ShellCommand,
    workflow::Workflow,
};

/// Summary name used when every workflow in the repository is executed.
pub const ALL_WORKFLOWS_SUMMARY_NAME: &str = "all-workflows";

/// Directory the repository is mounted at inside the job container.
const CONTAINER_WORKSPACE: &str = "/workspace";

/// File the container writes `GITHUB_PATH` additions to.
const GITHUB_PATH_FILE: &str = "/workspace/.github_path";

/// File the container writes `GITHUB_ENV` additions to.
const GITHUB_ENV_FILE: &str = "/workspace/.github_env";

/// Search order for workflow directories, so a Forgejo repository is detected
/// before falling back to the GitHub layout.
const WORKFLOW_DIRECTORIES: [&str; 2] = [".forgejo/workflows", ".github/workflows"];

/// Application service that executes workflows natively in containers.
///
/// Parses the workflow YAML, plans the job execution DAG, and runs each job
/// inside an ephemeral container created through the [`ContainerRuntimePort`].
/// Shell steps run directly; a step that references an action is handed to the
/// rest of the system as a [`DomainEvent::ActionExecutionRequested`] event, so
/// this service never depends on another inbound port to resolve or fetch
/// actions.
pub struct RunActService<R: ContainerRuntimePort, M: ImageMapperPort, E: EventPublisherPort> {
    runtime: R,
    image_mapper: M,
    event_publisher: E,
}

impl<R: ContainerRuntimePort, M: ImageMapperPort, E: EventPublisherPort> RunActService<R, M, E> {
    pub fn new(runtime: R, image_mapper: M, event_publisher: E) -> Self {
        Self {
            runtime,
            image_mapper,
            event_publisher,
        }
    }

    /// Returns every `.yml`/`.yaml` file directly inside `dir`, sorted by path.
    fn workflow_files_in(dir: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let mut files = Vec::new();
        for entry in read_dir(dir)? {
            let path = entry?.path();
            if path
                .extension()
                .is_some_and(|ext| ext == "yml" || ext == "yaml")
            {
                files.push(path);
            }
        }
        files.sort();
        Ok(files)
    }

    /// Finds the workflow file to execute.
    ///
    /// If the config specifies a workflow path, uses it directly. Otherwise
    /// auto-detects the CI platform by checking `.forgejo/workflows/` first,
    /// then `.github/workflows/`, and returns the first `.yml` file found.
    fn find_workflow(
        &self,
        config: &ActRunConfig,
        repo_path: &Path,
    ) -> Result<PathBuf, Box<dyn Error>> {
        if let Some(wf) = config.workflow() {
            let direct = repo_path.join(wf.as_str());
            if direct.exists() {
                return Ok(direct);
            }
            for platform_dir in &WORKFLOW_DIRECTORIES {
                let path = repo_path.join(platform_dir).join(wf.as_str());
                if path.exists() {
                    return Ok(path);
                }
            }
            return Err(format!("workflow file not found: {}", wf.as_str()).into());
        }

        for platform_dir in &WORKFLOW_DIRECTORIES {
            let workflows_dir = repo_path.join(platform_dir);
            if workflows_dir.exists() {
                return match Self::workflow_files_in(&workflows_dir)?.into_iter().next() {
                    Some(path) => Ok(path),
                    None => Err(format!("no workflow files found in {}/", platform_dir).into()),
                };
            }
        }

        Err("no workflows directory found (.forgejo/workflows/ or .github/workflows/)".into())
    }

    /// Returns every workflow file in the repository, `.forgejo` before `.github`.
    fn find_all_workflows(&self, repo_path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let mut workflows = Vec::new();
        for platform_dir in &WORKFLOW_DIRECTORIES {
            let workflows_dir = repo_path.join(platform_dir);
            if workflows_dir.exists() {
                workflows.extend(Self::workflow_files_in(&workflows_dir)?);
            }
        }
        if workflows.is_empty() {
            return Err(
                "no workflow files found in .forgejo/workflows/ or .github/workflows/".into(),
            );
        }
        Ok(workflows)
    }

    /// Merges workflow-level and job-level environment variables.
    fn build_env(
        workflow: &Workflow,
        job_env: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut env = workflow.env.clone();
        for (k, v) in job_env {
            env.insert(k.clone(), v.clone());
        }
        env
    }

    /// Builds the expression context a run's steps are evaluated against,
    /// populating the `secrets`, `inputs`, `github`, and `runner` contexts from
    /// the run configuration.
    fn build_context(config: &ActRunConfig, repository: &Repository) -> EvalContext {
        let secrets: Map<String, Value> = config
            .secrets()
            .iter()
            .map(|secret| {
                (
                    secret.name().to_string(),
                    Value::String(secret.value().into()),
                )
            })
            .collect();
        let inputs: Map<String, Value> = config
            .inputs()
            .iter()
            .map(|input| (input.key().to_string(), Value::String(input.value().into())))
            .collect();
        let event_name = config
            .event()
            .map_or("workflow_dispatch", |event| event.as_str());

        let mut event = Map::new();
        event.insert("inputs".into(), Value::Object(inputs.clone()));

        let mut github = Map::new();
        github.insert("event_name".into(), Value::String(event_name.into()));
        github.insert(
            "repository".into(),
            Value::String(repository.name().as_str().into()),
        );
        github.insert(
            "workspace".into(),
            Value::String(CONTAINER_WORKSPACE.into()),
        );
        github.insert("event".into(), Value::Object(event));

        let mut runner = Map::new();
        runner.insert("os".into(), Value::String("Linux".into()));
        runner.insert("arch".into(), Value::String("X64".into()));
        runner.insert("temp".into(), Value::String("/tmp".into()));

        let mut context = EvalContext::new();
        context.secrets = Value::Object(secrets);
        context.inputs = Value::Object(inputs);
        context.github = Value::Object(github);
        context.runner = Value::Object(runner);
        context
    }

    /// Returns a copy of `context` whose `env` context mirrors the environment
    /// the next step will run with.
    fn context_with_env(context: &EvalContext, env: &HashMap<String, String>) -> EvalContext {
        let mut step_context = context.clone();
        step_context.env = Value::Object(
            env.iter()
                .map(|(key, value)| (key.clone(), Value::String(value.clone())))
                .collect(),
        );
        step_context
    }

    /// Runs one planned job inside a fresh ephemeral container.
    ///
    /// Returns the job summary and the name of the container that was created.
    fn execute_run(
        &self,
        run: &Run,
        workflow: &Workflow,
        repo_path: &Path,
        context: &EvalContext,
    ) -> Result<(JobSummary, String), Box<dyn Error>> {
        let runs_on = run.job.runs_on.as_deref().unwrap_or("ubuntu-latest");
        let mut image = self.image_mapper.map(runs_on);

        if self.runtime.pull_image(&image, None).is_err() {
            image = self.image_mapper.fallback();
            self.runtime
                .pull_image(&image, None)
                .map_err(|e| format!("{:?}", e))?;
        }

        let mut container_env = Self::build_env(workflow, &run.job.env);
        container_env.insert("GITHUB_PATH".into(), GITHUB_PATH_FILE.into());
        container_env.insert("GITHUB_ENV".into(), GITHUB_ENV_FILE.into());
        container_env.insert("GITHUB_WORKSPACE".into(), CONTAINER_WORKSPACE.into());
        container_env.entry("PATH".to_string()).or_insert_with(|| {
            "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string()
        });
        let container_name = format!("ephemeral-act-{}-{}", run.job_id, process::id());
        let legacy_name = format!("ephemeral-act-{}", run.job_id);
        let _ = self.runtime.remove_container(&legacy_name);
        let _ = self.runtime.remove_container(&container_name);
        let container_config = ContainerConfig {
            image: image.clone(),
            platform: None,
            env: HashMap::new(),
            binds: vec![format!("{}:{}:Z", repo_path.display(), CONTAINER_WORKSPACE)],
            workdir: Some(CONTAINER_WORKSPACE.into()),
            cmd: Some(vec!["sleep".into(), "infinity".into()]),
            entrypoint: None,
            network: None,
            name: Some(container_name.clone()),
            runner_context: RunnerContext::default(),
        };

        let container: Arc<dyn ContainerPort> = Arc::from(
            self.runtime
                .create_container(&container_config)
                .map_err(|e| format!("{:?}", e))?,
        );

        let mut step_env = container_env.clone();
        let mut extra_path: Vec<String> = Vec::new();

        let mut job_success = true;
        let mut steps: Vec<StepSummary> = Vec::new();
        for step in &run.job.steps {
            let path = if extra_path.is_empty() {
                step_env.get("PATH").cloned().unwrap_or_default()
            } else {
                let base = step_env.get("PATH").cloned().unwrap_or_default();
                format!("{}:{}", extra_path.join(":"), base)
            };
            step_env.insert("PATH".into(), path);

            let step_type = step.step_type();
            let continue_on_error = step.continues_on_error();
            let step_started_at = Instant::now();
            let step_context = Self::context_with_env(context, &step_env);

            let outcome = StepInterpolator::interpolate(step, &step_context)
                .map_err(|error| {
                    StepError::new(format!("failed to resolve expressions: {error:?}"))
                })
                .and_then(|interpolated| {
                    self.execute_step(
                        &interpolated,
                        &step_context,
                        container.clone(),
                        repo_path,
                        &step_env,
                    )
                    .map(|response| (interpolated, response))
                });

            let (exit_code, stdout, stderr, label) = match outcome {
                Ok((interpolated, response)) => {
                    if response.exit_code != 0 && !continue_on_error {
                        job_success = false;
                    }
                    (
                        Some(response.exit_code),
                        response.stdout,
                        response.stderr,
                        Self::step_label(&interpolated),
                    )
                }
                Err(error) => {
                    if !continue_on_error {
                        job_success = false;
                    }
                    (
                        None,
                        error.stdout,
                        format!("step error: {}\n{}", error.message, error.stderr),
                        Self::step_label(step),
                    )
                }
            };
            steps.push(StepSummary {
                name: label,
                step_type,
                exit_code,
                continue_on_error,
                duration: step_started_at.elapsed(),
                stdout,
                stderr,
            });

            if let Ok(output) = container.exec(
                &["cat".into(), GITHUB_PATH_FILE.into()],
                None,
                &HashMap::new(),
            ) {
                for line in output.stdout.lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        extra_path.push(trimmed.to_string());
                    }
                }
            }
            if let Ok(output) = container.exec(
                &["cat".into(), GITHUB_ENV_FILE.into()],
                None,
                &HashMap::new(),
            ) {
                for line in output.stdout.lines() {
                    let trimmed = line.trim();
                    if let Some((key, value)) = trimmed.split_once('=') {
                        step_env.insert(key.to_string(), value.to_string());
                    }
                }
            }
        }

        Ok((
            JobSummary {
                job_id: run.job_id.clone(),
                name: run.job.name.clone(),
                steps,
                success: job_success,
            },
            container_name,
        ))
    }

    /// Runs a single step: shell scripts directly, action references by asking
    /// the rest of the system to execute them.
    fn execute_step(
        &self,
        step: &crate::core::workflow::Step,
        context: &EvalContext,
        container: Arc<dyn ContainerPort>,
        repo_path: &Path,
        env: &HashMap<String, String>,
    ) -> Result<ExecuteActionResponse, StepError> {
        if let Some(action_ref) = step.uses() {
            return self.request_action_execution(ExecuteActionRequest {
                action_ref: action_ref.to_string(),
                step: step.clone(),
                repo_path: repo_path.to_path_buf(),
                env: env.clone(),
                context: context.clone(),
                container,
            });
        }

        let command = ShellCommand::for_step(step, env)
            .ok_or_else(|| StepError::new("step has neither `run` nor `uses` defined"))?;

        container
            .exec(command.argv(), command.working_directory(), command.env())
            .map(|result| ExecuteActionResponse {
                exit_code: result.exit_code,
                stdout: result.stdout,
                stderr: result.stderr,
            })
            .map_err(|error| StepError::new(format!("{error:?}")))
    }

    /// Publishes the action execution request and returns the outcome reported
    /// by whichever handler ran the action.
    fn request_action_execution(
        &self,
        request: ExecuteActionRequest,
    ) -> Result<ExecuteActionResponse, StepError> {
        let action_ref = request.action_ref.clone();
        let outcomes = self
            .event_publisher
            .publish(DomainEvent::ActionExecutionRequested(Box::new(
                ActionExecutionRequestedPayload { request },
            )));

        outcomes
            .into_iter()
            .map(|outcome| match outcome {
                EventOutcome::ActionExecuted(result) => result,
            })
            .next()
            .unwrap_or_else(|| {
                Err(StepError::new(format!(
                    "no handler executed the action '{action_ref}'"
                )))
            })
    }

    fn step_label(step: &crate::core::workflow::Step) -> String {
        step.name
            .as_deref()
            .or(step.id.as_deref())
            .or(step.run.as_deref())
            .or(step.uses.as_deref())
            .unwrap_or("unnamed step")
            .to_string()
    }

    /// Runs every job of a single workflow file.
    fn execute_workflow(
        &self,
        workflow_path: &Path,
        repo_path: &Path,
        context: &EvalContext,
    ) -> Result<WorkflowExecution, Box<dyn Error>> {
        let yaml = read_to_string(workflow_path)?;
        let workflow: Workflow = serde_yaml::from_str(&yaml)?;
        let workflow_name = workflow.name.clone().unwrap_or_else(|| "unnamed".into());
        let plan = Planner.plan(&workflow).map_err(|e| format!("{:?}", e))?;

        let mut job_summaries: Vec<JobSummary> = Vec::new();
        let mut container_names: Vec<String> = Vec::new();
        let mut success = true;

        for stage in &plan.stages {
            for run in &stage.runs {
                let (job_summary, container_name) =
                    self.execute_run(run, &workflow, repo_path, context)?;
                success &= job_summary.success;
                job_summaries.push(job_summary);
                container_names.push(container_name);
            }
        }

        Ok(WorkflowExecution {
            workflow_name,
            job_summaries,
            container_names,
            success,
        })
    }

    /// Runs every workflow file in the repository sequentially.
    fn execute_every_workflow(
        &self,
        repo_path: &Path,
        context: &EvalContext,
    ) -> Result<WorkflowExecution, Box<dyn Error>> {
        let mut merged = WorkflowExecution {
            workflow_name: ALL_WORKFLOWS_SUMMARY_NAME.into(),
            job_summaries: Vec::new(),
            container_names: Vec::new(),
            success: true,
        };

        for workflow_path in self.find_all_workflows(repo_path)? {
            let WorkflowExecution {
                workflow_name,
                job_summaries,
                container_names,
                success,
            } = self.execute_workflow(&workflow_path, repo_path, context)?;

            merged
                .job_summaries
                .extend(job_summaries.into_iter().map(|mut job| {
                    job.name = job.name.map(|name| format!("{} / {}", workflow_name, name));
                    job
                }));
            merged.container_names.extend(container_names);
            merged.success &= success;
        }

        Ok(merged)
    }
}

impl<R: ContainerRuntimePort, M: ImageMapperPort, E: EventPublisherPort> RunActPort
    for RunActService<R, M, E>
{
    fn execute(&self, request: RunActRequest) -> Result<RunSummary, Box<dyn Error>> {
        let RunActRequest { config, repository } = request;
        let repo_path = repository.path().as_path();
        let started_at = Instant::now();
        let context = Self::build_context(&config, &repository);

        let execution = if config.all_workflows() {
            self.execute_every_workflow(repo_path, &context)?
        } else {
            self.execute_workflow(
                &self.find_workflow(&config, repo_path)?,
                repo_path,
                &context,
            )?
        };

        self.event_publisher
            .publish(DomainEvent::ActRunCompleted(ActRunCompletedPayload {
                container_names: execution.container_names,
                success: execution.success,
            }));

        Ok(RunSummary {
            name: execution.workflow_name,
            job_summaries: execution.job_summaries,
            success: execution.success,
            duration: started_at.elapsed(),
        })
    }
}
