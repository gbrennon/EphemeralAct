use std::fs;

use ephact::application::{
    dtos::DetectWorkflowFileRequest,
    ports::outbound::detect_workflow_file_port::DetectWorkflowFilePort,
    services::{
        detect_workflow_file_service::DetectWorkflowFileService,
        list_workflow_directory_service::ListWorkflowDirectoryService,
    },
};

fn service() -> DetectWorkflowFileService {
    DetectWorkflowFileService::new(Box::new(ListWorkflowDirectoryService::new()))
}

#[test]
fn execute_returns_the_first_forgejo_workflow_file() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join(".forgejo/workflows")).unwrap();
    fs::write(tmp.path().join(".forgejo/workflows/b.yml"), "").unwrap();
    fs::write(tmp.path().join(".forgejo/workflows/a.yml"), "").unwrap();

    let path = service()
        .execute(DetectWorkflowFileRequest {
            repo_path: tmp.path(),
        })
        .unwrap();

    assert_eq!(path, tmp.path().join(".forgejo/workflows/a.yml"));
}

#[test]
fn execute_prefers_forgejo_over_github() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join(".forgejo/workflows")).unwrap();
    fs::create_dir_all(tmp.path().join(".github/workflows")).unwrap();
    fs::write(tmp.path().join(".forgejo/workflows/forgejo.yml"), "").unwrap();
    fs::write(tmp.path().join(".github/workflows/github.yml"), "").unwrap();

    let path = service()
        .execute(DetectWorkflowFileRequest {
            repo_path: tmp.path(),
        })
        .unwrap();

    assert_eq!(path, tmp.path().join(".forgejo/workflows/forgejo.yml"));
}

#[test]
fn execute_errors_for_an_empty_forgejo_directory_without_trying_github() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join(".forgejo/workflows")).unwrap();
    fs::create_dir_all(tmp.path().join(".github/workflows")).unwrap();
    fs::write(tmp.path().join(".github/workflows/github.yml"), "").unwrap();

    let error = service()
        .execute(DetectWorkflowFileRequest {
            repo_path: tmp.path(),
        })
        .unwrap_err()
        .to_string();

    assert_eq!(error, "no workflow files found in .forgejo/workflows/");
}

#[test]
fn execute_errors_when_the_repository_has_no_workflows_directory() {
    let tmp = tempfile::tempdir().unwrap();

    let error = service()
        .execute(DetectWorkflowFileRequest {
            repo_path: tmp.path(),
        })
        .unwrap_err()
        .to_string();

    assert_eq!(
        error,
        "no workflows directory found (.forgejo/workflows/ or .github/workflows/)"
    );
}
