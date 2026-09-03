use std::fs;

use ephact::application::{
    dtos::ListAllWorkflowFilesRequest,
    ports::outbound::list_all_workflow_files_port::ListAllWorkflowFilesPort,
    services::{
        list_all_workflow_files_service::ListAllWorkflowFilesService,
        list_workflow_directory_service::ListWorkflowDirectoryService,
    },
};

fn service() -> ListAllWorkflowFilesService {
    ListAllWorkflowFilesService::new(Box::new(ListWorkflowDirectoryService::new()))
}

#[test]
fn execute_returns_files_from_both_platform_directories_forgejo_first() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join(".forgejo/workflows")).unwrap();
    fs::create_dir_all(tmp.path().join(".github/workflows")).unwrap();
    fs::write(tmp.path().join(".forgejo/workflows/alpha.yml"), "").unwrap();
    fs::write(tmp.path().join(".github/workflows/beta.yml"), "").unwrap();

    let response = service()
        .execute(ListAllWorkflowFilesRequest {
            repo_path: tmp.path(),
        })
        .unwrap();

    assert_eq!(
        response.workflow_files,
        vec![
            tmp.path().join(".forgejo/workflows/alpha.yml"),
            tmp.path().join(".github/workflows/beta.yml")
        ]
    );
}

#[test]
fn execute_returns_github_files_when_the_repository_has_only_those() {
    let tmp = tempfile::tempdir().unwrap();
    fs::create_dir_all(tmp.path().join(".github/workflows")).unwrap();
    fs::write(tmp.path().join(".github/workflows/beta.yml"), "").unwrap();

    let response = service()
        .execute(ListAllWorkflowFilesRequest {
            repo_path: tmp.path(),
        })
        .unwrap();

    assert_eq!(
        response.workflow_files,
        vec![tmp.path().join(".github/workflows/beta.yml")]
    );
}

#[test]
fn execute_errors_when_the_repository_holds_no_workflow_files() {
    let tmp = tempfile::tempdir().unwrap();

    let error = service()
        .execute(ListAllWorkflowFilesRequest {
            repo_path: tmp.path(),
        })
        .unwrap_err()
        .to_string();

    assert_eq!(
        error,
        "no workflow files found in .forgejo/workflows/ or .github/workflows/"
    );
}
