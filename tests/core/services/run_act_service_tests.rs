mod common;

use std::path::Path;

use ephemeral_act::core::{
    ports::inbound::RunActUseCase,
    services::run_act_service::RunActService,
    ActRunConfig, ActWorkflow, RepoPath, Repository, RepositoryName,
};

use crate::common::{FakeEventPublisher, FakeImageMapper, FakeRuntime};

fn make_repo(path: &Path) -> Repository {
    let git_dir = path.join(".git");
    if !git_dir.exists() {
        std::fs::create_dir_all(&git_dir).ok();
    }
    let repo_path = RepoPath::new(path.to_path_buf()).unwrap();
    let name = RepositoryName::new("test-repo".into()).unwrap();
    Repository::new(repo_path, name)
}

#[test]
fn run_act_executes_workflow_and_publishes_event() {
    let repo = make_repo(Path::new(env!("CARGO_MANIFEST_DIR")));
    let runtime = FakeRuntime::new();
    let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());
    let config = ActRunConfig::new();
    let result = service.run_act(config, repo).unwrap();
    assert!(result.success);
}

#[test]
fn run_act_finds_workflow_in_forgejo_dir() {
    let repo = make_repo(Path::new(env!("CARGO_MANIFEST_DIR")));
    let runtime = FakeRuntime::new();
    let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());
    let config =
        ActRunConfig::new().with_workflow(ActWorkflow::new(".forgejo/workflows/ci.yml".into()));
    let result = service.run_act(config, repo).unwrap();
    assert!(result.success);
}

#[test]
fn run_act_errors_on_nonexistent_workflow() {
    let repo = make_repo(Path::new(env!("CARGO_MANIFEST_DIR")));
    let runtime = FakeRuntime::new();
    let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());
    let config = ActRunConfig::new().with_workflow(ActWorkflow::new("nonexistent.yml".into()));
    let err = service.run_act(config, repo).unwrap_err();
    assert!(err.to_string().contains("nonexistent.yml"), "{}", err);
}

#[test]
fn run_act_errors_when_no_workflow_found() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = make_repo(tmp.path());
    let runtime = FakeRuntime::new();
    let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());
    let config = ActRunConfig::new();
    let err = service.run_act(config, repo).unwrap_err();
    assert!(err.to_string().contains("workflows directory"), "{}", err);
}

#[test]
fn run_act_reports_failure_on_step_error() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join(".forgejo/workflows")).unwrap();
    std::fs::write(
        tmp.path().join(".forgejo/workflows/fail.yml"),
        "name: Fail\non: push\njobs:\n  fail:\n    runs-on: ubuntu-latest\n    steps:\n      - run: exit 1\n",
    )
    .unwrap();
    let repo = make_repo(tmp.path());
    let runtime = FakeRuntime::new();
    runtime
        .exec_results
        .borrow_mut()
        .push(ephemeral_act::core::ports::outbound::ExecResult {
            exit_code: 1,
            stdout: String::new(),
            stderr: "fail".into(),
        });
    let service = RunActService::new(runtime, FakeImageMapper, FakeEventPublisher::new());
    let config = ActRunConfig::new();
    let result = service.run_act(config, repo).unwrap();
    assert!(!result.success);
}
