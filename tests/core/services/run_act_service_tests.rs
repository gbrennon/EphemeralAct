mod fake_container;
mod fake_event_publisher;
mod fake_runtime;
mod fake_image_mapper;

use std::path::Path;

use ephemeral_act::core::{
    ActRunConfig, Repository, RunActUseCase,
    ports::outbound::ExecResult,
    services::run_act_service::RunActService,
    value_objects::{ActWorkflow, RepoPath, RepositoryName},
};
use fake_event_publisher::FakeEventPublisher;
use fake_runtime::FakeRuntime;
use fake_image_mapper::FakeImageMapper;

fn make_repo(path: &Path) -> Repository {
    let repo_path = RepoPath::new(path.to_path_buf()).unwrap();
    let name = RepositoryName::new("test-repo".into()).unwrap();
    Repository::new(repo_path, name)
}

fn init_git(dir: &Path) {
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(dir)
        .output()
        .unwrap();
}

fn setup_workflow_dir() -> (tempfile::TempDir, Repository) {
    let dir = tempfile::tempdir().unwrap();
    init_git(dir.path());
    let workflows = dir.path().join(".github").join("workflows");
    std::fs::create_dir_all(&workflows).unwrap();
    std::fs::write(
        workflows.join("ci.yml"),
        r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: echo hello
"#,
    )
    .unwrap();
    let repo = make_repo(dir.path());
    (dir, repo)
}

#[test]
fn pulls_image_and_creates_container_for_job() {
    let (_dir, repo) = setup_workflow_dir();
    let runtime = FakeRuntime::new();
    let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());

    let config = ActRunConfig::new();
    let result = service.run_act(config, repo).unwrap();

    assert!(result.success);
    assert!(result.stdout.contains("hello"));
}

#[test]
fn maps_runs_on_to_container_image() {
    let dir = tempfile::tempdir().unwrap();
    init_git(dir.path());
    let workflows = dir.path().join(".github").join("workflows");
    std::fs::create_dir_all(&workflows).unwrap();
    std::fs::write(
        workflows.join("ci.yml"),
        r#"
name: CI
on: push
jobs:
  build:
    runs-on: ubuntu-22.04
    steps:
      - run: echo hello
"#,
    )
    .unwrap();
    let repo = make_repo(dir.path());

    let runtime = FakeRuntime::new();
    let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());

    let config = ActRunConfig::new();
    service.run_act(config, repo).unwrap();
}

#[test]
fn uses_specified_workflow_file() {
    let dir = tempfile::tempdir().unwrap();
    init_git(dir.path());
    let workflows = dir.path().join(".github").join("workflows");
    std::fs::create_dir_all(&workflows).unwrap();
    std::fs::write(
        workflows.join("custom.yml"),
        r#"
name: Custom
on: push
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: echo custom
"#,
    )
    .unwrap();
    let repo = make_repo(dir.path());

    let runtime = FakeRuntime::new();
    let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());

    let config =
        ActRunConfig::new().with_workflow(ActWorkflow::new(".github/workflows/custom.yml".into()));
    let result = service.run_act(config, repo).unwrap();

    assert!(result.success);
    assert!(result.stdout.contains("custom"));
}

#[test]
fn errors_when_workflow_file_not_found() {
    let dir = tempfile::tempdir().unwrap();
    init_git(dir.path());
    let repo = make_repo(dir.path());

    let runtime = FakeRuntime::new();
    let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());

    let config = ActRunConfig::new().with_workflow(ActWorkflow::new("nonexistent.yml".into()));
    let err = service.run_act(config, repo).unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[test]
fn errors_when_no_workflows_directory() {
    let dir = tempfile::tempdir().unwrap();
    init_git(dir.path());
    let repo = make_repo(dir.path());

    let runtime = FakeRuntime::new();
    let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());

    let config = ActRunConfig::new();
    let err = service.run_act(config, repo).unwrap_err();
    assert!(err.to_string().contains("workflows"));
}

#[test]
fn propagates_step_failure() {
    let (_dir, repo) = setup_workflow_dir();
    let runtime = FakeRuntime::new();
    runtime.exec_results.borrow_mut().push(ExecResult {
        exit_code: 1,
        stdout: String::new(),
        stderr: "command failed".into(),
    });
    let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());

    let config = ActRunConfig::new();
    let result = service.run_act(config, repo).unwrap();

    assert!(!result.success);
    assert!(result.stderr.contains("command failed"));
}
