use std::fs;

use ephact::application::{
    dtos::ListActionsRequest, ports::inbound::list_actions_port::ListActionsPort,
    services::list_actions_service::ListActionsService,
};

use crate::common::fakes::fake_workflow_file_parser::FakeWorkflowFileParser;

fn write_workflow(dir: &std::path::Path, name: &str, body: &str) {
    fs::create_dir_all(dir.join(".forgejo/workflows")).unwrap();
    fs::write(dir.join(".forgejo/workflows").join(name), body).unwrap();
}

fn write_github_workflow(dir: &std::path::Path, name: &str, body: &str) {
    fs::create_dir_all(dir.join(".github/workflows")).unwrap();
    fs::write(dir.join(".github/workflows").join(name), body).unwrap();
}

#[test]
fn list_when_yaml_has_uses_actions() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_path = tmp.path();

    write_workflow(
        repo_path,
        "ci.yml",
        "jobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: ./local-action\n",
    );

    let service = ListActionsService::new(Box::new(FakeWorkflowFileParser::new()));
    let request = ListActionsRequest::new(repo_path.into());
    let response = service.execute(request).unwrap();

    assert!(
        response
            .actions
            .contains(&"actions/checkout@v4".to_string())
    );
    assert!(response.actions.contains(&"./local-action".to_string()));
    assert_eq!(response.actions.len(), 2);
}

#[test]
fn list_when_multiple_workflows_with_actions() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_path = tmp.path();

    write_workflow(
        repo_path,
        "ci.yml",
        "jobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n",
    );
    write_workflow(
        repo_path,
        "deploy.yml",
        "jobs:\n  deploy:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/setup-node@v3\n",
    );

    let service = ListActionsService::new(Box::new(FakeWorkflowFileParser::new()));
    let request = ListActionsRequest::new(repo_path.into());
    let response = service.execute(request).unwrap();

    assert!(
        response
            .actions
            .contains(&"actions/checkout@v4".to_string())
    );
    assert!(
        response
            .actions
            .contains(&"actions/setup-node@v3".to_string())
    );
    assert_eq!(response.actions.len(), 2);
}

#[test]
fn list_when_github_workflows_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_path = tmp.path();

    write_github_workflow(
        repo_path,
        "build.yml",
        "jobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n",
    );

    let service = ListActionsService::new(Box::new(FakeWorkflowFileParser::new()));
    let request = ListActionsRequest::new(repo_path.into());
    let response = service.execute(request).unwrap();

    assert!(
        response
            .actions
            .contains(&"actions/checkout@v4".to_string())
    );
    assert_eq!(response.actions.len(), 1);
}

#[test]
fn list_duplicate_actions_dedup() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_path = tmp.path();

    write_workflow(
        repo_path,
        "ci.yml",
        "jobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n      - uses: actions/checkout@v4\n",
    );

    let service = ListActionsService::new(Box::new(FakeWorkflowFileParser::new()));
    let request = ListActionsRequest::new(repo_path.into());
    let response = service.execute(request).unwrap();

    assert_eq!(response.actions.len(), 1);
    assert!(
        response
            .actions
            .contains(&"actions/checkout@v4".to_string())
    );
}

#[test]
fn list_when_no_uses() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_path = tmp.path();

    write_workflow(
        repo_path,
        "ci.yml",
        "jobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hello\n",
    );

    let service = ListActionsService::new(Box::new(FakeWorkflowFileParser::new()));
    let request = ListActionsRequest::new(repo_path.into());
    let response = service.execute(request).unwrap();

    assert!(response.actions.is_empty());
}

#[test]
fn list_when_repository_has_no_workflows() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_path = tmp.path();

    // No workflow directories
    let service = ListActionsService::new(Box::new(FakeWorkflowFileParser::new()));
    let request = ListActionsRequest::new(repo_path.into());
    let response = service.execute(request).unwrap();

    assert!(response.actions.is_empty());
}

#[test]
fn list_when_path_is_invalid() {
    let service = ListActionsService::new(Box::new(FakeWorkflowFileParser::new()));
    let request = ListActionsRequest::new(std::path::PathBuf::from("/nonexistent"));
    let response = service.execute(request).unwrap();

    assert!(response.actions.is_empty());
}
