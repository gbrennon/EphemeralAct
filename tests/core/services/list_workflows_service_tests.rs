use std::fs;

use ephemeral_act::core::{
    dtos::ListWorkflowsRequest, services::list_workflows_service::ListWorkflowsService,
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
fn list_when_repository_has_workflows() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_path = tmp.path();

    write_workflow(
        repo_path,
        "ci.yml",
        "name: CI\non: [push]\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hello\n",
    );

    let parser = FakeWorkflowFileParser::new();
    let request = ListWorkflowsRequest::new(repo_path.into());
    let response = ListWorkflowsService::execute(request, &parser).unwrap();

    assert_eq!(response.workflows.len(), 1);
    assert_eq!(response.workflows[0].name, Some("CI".to_string()));
    assert_eq!(response.workflows[0].file, Some("workflow.yml".to_string()));
}

#[test]
fn list_when_multiple_workflows() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_path = tmp.path();

    write_workflow(
        repo_path,
        "ci.yml",
        "name: CI\non: [push]\njobs:\n  test:\n    runs-on: ubuntu-latest\n",
    );
    write_workflow(
        repo_path,
        "deploy.yml",
        "name: Deploy\non: [release]\njobs:\n  deploy:\n    runs-on: ubuntu-latest\n",
    );

    let parser = FakeWorkflowFileParser::new();
    let request = ListWorkflowsRequest::new(repo_path.into());
    let response = ListWorkflowsService::execute(request, &parser).unwrap();

    assert_eq!(response.workflows.len(), 2);
}

#[test]
fn list_when_github_workflows_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_path = tmp.path();

    write_github_workflow(
        repo_path,
        "build.yml",
        "name: Build\non: [push]\njobs:\n  build:\n    runs-on: ubuntu-latest\n",
    );

    let parser = FakeWorkflowFileParser::new();
    let request = ListWorkflowsRequest::new(repo_path.into());
    let response = ListWorkflowsService::execute(request, &parser).unwrap();

    assert_eq!(response.workflows.len(), 1);
    assert_eq!(response.workflows[0].name, Some("Build".to_string()));
}

#[test]
fn list_when_no_name_falls_back_to_none() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_path = tmp.path();

    write_workflow(
        repo_path,
        "ci.yml",
        "on: [push]\njobs:\n  test:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hello\n",
    );

    let parser = FakeWorkflowFileParser::new();
    let request = ListWorkflowsRequest::new(repo_path.into());
    let response = ListWorkflowsService::execute(request, &parser).unwrap();

    assert_eq!(response.workflows.len(), 1);
    assert!(response.workflows[0].name.is_none());
}

#[test]
fn list_when_repository_has_no_workflows() {
    let tmp = tempfile::tempdir().unwrap();
    let repo_path = tmp.path();

    // No workflow directories
    let parser = FakeWorkflowFileParser::new();
    let request = ListWorkflowsRequest::new(repo_path.into());
    let response = ListWorkflowsService::execute(request, &parser).unwrap();

    assert!(response.workflows.is_empty());
}

#[test]
fn list_when_path_is_invalid() {
    let parser = FakeWorkflowFileParser::new();
    let request = ListWorkflowsRequest::new(std::path::PathBuf::from("/nonexistent"));
    let response = ListWorkflowsService::execute(request, &parser).unwrap();

    assert!(response.workflows.is_empty());
}
