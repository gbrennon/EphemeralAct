use std::{
    collections::HashMap,
    error::Error,
    fs::{read_dir, read_to_string},
    path::{Path, PathBuf},
    process,
    time::Instant,
};

use crate::core::{
    ActRunConfig,
    dtos::{JobSummary, RunActRequest, RunSummary, StepSummary},
    events::{ActRunCompletedPayload, DomainEvent},
    planner::Planner,
    ports::{
        inbound::run_act_port::RunActPort,
        outbound::{
            ContainerConfig, ContainerRuntimePort, EventPublisherPort, ImageMapperPort,
            RunnerContext,
        },
    },
    services::step_runner_service::StepRunnerService,
    workflow::Workflow,
};

/// Application service that executes GitHub Actions workflows natively in
/// containers.
///
/// Parses the workflow YAML, plans the job execution DAG, and runs each job
/// inside an ephemeral container using the provided [`ContainerRuntimePort`].
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
            for platform_dir in &[".forgejo/workflows", ".github/workflows"] {
                let path = repo_path.join(platform_dir).join(wf.as_str());
                if path.exists() {
                    return Ok(path);
                }
            }
            return Err(format!("workflow file not found: {}", wf.as_str()).into());
        }

        for platform_dir in &[".forgejo/workflows", ".github/workflows"] {
            let workflows_dir = repo_path.join(platform_dir);
            if workflows_dir.exists() {
                for entry in read_dir(&workflows_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path
                        .extension()
                        .is_some_and(|ext| ext == "yml" || ext == "yaml")
                    {
                        return Ok(path);
                    }
                }
                return Err(format!("no workflow files found in {}/", platform_dir).into());
            }
        }

        Err("no workflows directory found (.forgejo/workflows/ or .github/workflows/)".into())
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
}

impl<R: ContainerRuntimePort, M: ImageMapperPort, E: EventPublisherPort> RunActPort
    for RunActService<R, M, E>
{
    fn execute(&self, request: RunActRequest) -> Result<RunSummary, Box<dyn Error>> {
        let RunActRequest { config, repository } = request;
        let repo_path = repository.path().as_path();

        let workflow_path = self.find_workflow(&config, repo_path)?;

        let yaml = read_to_string(&workflow_path)?;
        let workflow: Workflow = serde_yaml::from_str(&yaml)?;
        let workflow_name = workflow.name.clone().unwrap_or_else(|| "unnamed".into());

        let planner = Planner::new();
        let plan = planner.plan(&workflow).map_err(|e| format!("{:?}", e))?;

        let started_at = Instant::now();
        let mut job_summaries: Vec<JobSummary> = Vec::new();
        let mut success = true;

        let mut container_names: Vec<String> = Vec::new();

        for stage in &plan.stages {
            for run in &stage.runs {
                let runs_on = run.job.runs_on.as_deref().unwrap_or("ubuntu-latest");
                let mut image = self.image_mapper.map(runs_on);

                if self.runtime.pull_image(&image, None).is_err() {
                    image = self.image_mapper.fallback();
                    self.runtime
                        .pull_image(&image, None)
                        .map_err(|e| format!("{:?}", e))?;
                }

                let mut container_env = Self::build_env(&workflow, &run.job.env);
                let github_path = "/workspace/.github_path";
                let github_env = "/workspace/.github_env";
                container_env.insert("GITHUB_PATH".into(), github_path.into());
                container_env.insert("GITHUB_ENV".into(), github_env.into());
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
                    binds: vec![format!("{}:/workspace:Z", repo_path.display())],
                    workdir: Some("/workspace".into()),
                    cmd: Some(vec!["sleep".into(), "infinity".into()]),
                    entrypoint: None,
                    network: None,
                    name: Some(container_name.clone()),
                    runner_context: RunnerContext::default(),
                };

                let container = self
                    .runtime
                    .create_container(&container_config)
                    .map_err(|e| format!("{:?}", e))?;
                container_names.push(container_name);

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

                    let step_label = step
                        .name
                        .as_deref()
                        .or(step.id.as_deref())
                        .or(step.run.as_deref())
                        .or(step.uses.as_deref())
                        .unwrap_or("unnamed step");

                    let step_type = step.step_type();
                    let continue_on_error = step.continues_on_error();
                    let step_started_at = Instant::now();

                    let (exit_code, stdout, stderr) = match StepRunnerService::execute(
                        step,
                        container.as_ref(),
                        repo_path,
                        &step_env,
                    ) {
                        Ok(result) => {
                            if result.exit_code != 0 && !continue_on_error {
                                job_success = false;
                                success = false;
                            }
                            (Some(result.exit_code), result.stdout, result.stderr)
                        }
                        Err(e) => {
                            if !continue_on_error {
                                job_success = false;
                                success = false;
                            }
                            (
                                None,
                                e.stdout,
                                format!("step error: {}\n{}", e.message, e.stderr),
                            )
                        }
                    };
                    steps.push(StepSummary {
                        name: step_label.to_string(),
                        step_type,
                        exit_code,
                        continue_on_error,
                        duration: step_started_at.elapsed(),
                        stdout,
                        stderr,
                    });

                    if let Ok(output) =
                        container.exec(&["cat".into(), github_path.into()], None, &HashMap::new())
                    {
                        for line in output.stdout.lines() {
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                extra_path.push(trimmed.to_string());
                            }
                        }
                    }
                    if let Ok(output) =
                        container.exec(&["cat".into(), github_env.into()], None, &HashMap::new())
                    {
                        for line in output.stdout.lines() {
                            let trimmed = line.trim();
                            if let Some((key, value)) = trimmed.split_once('=') {
                                step_env.insert(key.to_string(), value.to_string());
                            }
                        }
                    }
                }
                job_summaries.push(JobSummary {
                    job_id: run.job_id.clone(),
                    name: run.job.name.clone(),
                    steps,
                    success: job_success,
                });
            }
        }

        self.event_publisher
            .publish(DomainEvent::ActRunCompleted(ActRunCompletedPayload {
                container_names,
                success,
            }));

        Ok(RunSummary {
            name: workflow_name,
            job_summaries,
            success,
            duration: started_at.elapsed(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;
    use crate::core::{
        Repository,
        ports::outbound::{
            ContainerError, ContainerPort, ExecResult, FileEntry, HostInfo, ImageMapperPort,
            RunnerContext,
        },
        value_objects::{ActWorkflow, RepoPath, RepositoryName},
    };

    /// Stub image mapper that passes platforms through unchanged.
    struct FakeImageMapper;

    impl ImageMapperPort for FakeImageMapper {
        fn map(&self, platform: &str) -> String {
            platform.to_string()
        }

        fn fallback(&self) -> String {
            "catthehacker/ubuntu:act-latest".into()
        }
    }

    struct FakeEventPublisher(RefCell<Vec<DomainEvent>>);

    impl FakeEventPublisher {
        fn new() -> Self {
            Self(RefCell::new(Vec::new()))
        }
    }

    impl EventPublisherPort for FakeEventPublisher {
        fn publish(&self, event: DomainEvent) {
            self.0.borrow_mut().push(event);
        }
    }
    struct FakeRuntime {
        pulled_images: RefCell<Vec<String>>,
        created_containers: RefCell<Vec<ContainerConfig>>,
        exec_results: RefCell<Vec<ExecResult>>,
        removed_containers: RefCell<Vec<String>>,
        stopped_containers: RefCell<Vec<String>>,
    }

    impl FakeRuntime {
        fn new() -> Self {
            Self {
                pulled_images: RefCell::new(vec![]),
                created_containers: RefCell::new(vec![]),
                exec_results: RefCell::new(vec![]),
                removed_containers: RefCell::new(vec![]),
                stopped_containers: RefCell::new(vec![]),
            }
        }
    }

    impl ContainerRuntimePort for FakeRuntime {
        fn pull_image(&self, image: &str, _platform: Option<&str>) -> Result<(), ContainerError> {
            self.pulled_images.borrow_mut().push(image.to_string());
            Ok(())
        }

        fn create_container(
            &self,
            config: &ContainerConfig,
        ) -> Result<Box<dyn ContainerPort>, ContainerError> {
            self.created_containers.borrow_mut().push(config.clone());
            Ok(Box::new(FakeContainer {
                exec_results: self.exec_results.clone(),
            }))
        }

        fn remove_container(&self, name: &str) -> Result<(), ContainerError> {
            self.removed_containers.borrow_mut().push(name.to_string());
            Ok(())
        }

        fn stop_container(&self, name: &str) -> Result<(), ContainerError> {
            self.stopped_containers.borrow_mut().push(name.to_string());
            Ok(())
        }

        fn get_host_info(&self) -> Result<HostInfo, ContainerError> {
            Ok(HostInfo {
                os: "linux".into(),
                arch: "amd64".into(),
                engine_version: "1.0".into(),
            })
        }
    }

    struct FakeContainer {
        exec_results: RefCell<Vec<ExecResult>>,
    }

    impl ContainerPort for FakeContainer {
        fn exec(
            &self,
            _cmd: &[String],
            _workdir: Option<&str>,
            _env: &HashMap<String, String>,
        ) -> Result<ExecResult, ContainerError> {
            Ok(self.exec_results.borrow_mut().pop().unwrap_or(ExecResult {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            }))
        }

        fn copy_to(
            &self,
            _container_path: &str,
            _entries: &[FileEntry],
        ) -> Result<(), ContainerError> {
            Ok(())
        }

        fn copy_from(&self, _container_path: &str) -> Result<Vec<FileEntry>, ContainerError> {
            Ok(vec![])
        }

        fn remove(&self) -> Result<(), ContainerError> {
            Ok(())
        }

        fn get_runner_context(&self) -> Result<RunnerContext, ContainerError> {
            Ok(RunnerContext::default())
        }
    }

    fn make_repo() -> Repository {
        let crate_root = std::env::var("CARGO_MANIFEST_DIR").unwrap();
        let path = RepoPath::new(crate_root).unwrap();
        let name = RepositoryName::new("test-repo".into()).unwrap();
        Repository::new(path, name)
    }

    #[test]
    fn find_workflow_uses_config_path_when_set() {
        let runtime = FakeRuntime::new();
        let event_publisher = FakeEventPublisher::new();
        let service = RunActService::new(runtime, FakeImageMapper, event_publisher);
        let repo = make_repo();

        let config = ActRunConfig::new().with_workflow(ActWorkflow::new("Cargo.toml".into()));

        let result = service
            .find_workflow(&config, repo.path().as_path())
            .unwrap();
        assert!(result.ends_with("Cargo.toml"));
    }

    #[test]
    fn find_workflow_errors_when_config_path_not_found() {
        let runtime = FakeRuntime::new();
        let event_publisher = FakeEventPublisher::new();
        let service = RunActService::new(runtime, FakeImageMapper, event_publisher);
        let repo = make_repo();

        let config = ActRunConfig::new().with_workflow(ActWorkflow::new("nonexistent.yml".into()));

        let err = service
            .find_workflow(&config, repo.path().as_path())
            .unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn build_env_merges_workflow_and_job_env() {
        let yaml = r#"
on: push
env:
  FOO: bar
  BAZ: qux
jobs:
  test:
    runs-on: ubuntu-latest
    steps: []
"#;
        let workflow: Workflow = serde_yaml::from_str(yaml).unwrap();
        let mut job_env = HashMap::new();
        job_env.insert("BAZ".into(), "overridden".into());
        job_env.insert("JOB_VAR".into(), "job_val".into());

        let env = RunActService::<FakeRuntime, FakeImageMapper, FakeEventPublisher>::build_env(
            &workflow, &job_env,
        );

        assert_eq!(env.get("FOO").map(|s| s.as_str()), Some("bar"));
        assert_eq!(env.get("BAZ").map(|s| s.as_str()), Some("overridden"));
        assert_eq!(env.get("JOB_VAR").map(|s| s.as_str()), Some("job_val"));
    }
}
