use std::fs;

use ephact::application::{
    dtos::ListWorkflowDirectoryRequest,
    ports::outbound::list_workflow_directory_port::ListWorkflowDirectoryPort,
    services::list_workflow_directory_service::ListWorkflowDirectoryService,
};

#[test]
fn execute_returns_yml_and_yaml_files_sorted_by_path() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("second.yaml"), "").unwrap();
    fs::write(tmp.path().join("first.yml"), "").unwrap();
    let service = ListWorkflowDirectoryService::new();

    let response = service
        .execute(ListWorkflowDirectoryRequest {
            directory: tmp.path(),
        })
        .unwrap();

    assert_eq!(
        response.workflow_files,
        vec![tmp.path().join("first.yml"), tmp.path().join("second.yaml")]
    );
}

#[test]
fn execute_excludes_other_extensions_and_subdirectories() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("ci.yml"), "").unwrap();
    fs::write(tmp.path().join("readme.md"), "").unwrap();
    fs::create_dir(tmp.path().join("nested")).unwrap();
    let service = ListWorkflowDirectoryService::new();

    let response = service
        .execute(ListWorkflowDirectoryRequest {
            directory: tmp.path(),
        })
        .unwrap();

    assert_eq!(response.workflow_files, vec![tmp.path().join("ci.yml")]);
}

#[test]
fn execute_errors_when_the_directory_is_missing() {
    let tmp = tempfile::tempdir().unwrap();
    let service = ListWorkflowDirectoryService::new();

    let result = service.execute(ListWorkflowDirectoryRequest {
        directory: &tmp.path().join("absent"),
    });

    assert!(result.is_err());
}

#[test]
fn execute_returns_no_files_for_an_empty_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let service = ListWorkflowDirectoryService::new();

    let response = service
        .execute(ListWorkflowDirectoryRequest {
            directory: tmp.path(),
        })
        .unwrap();

    assert!(response.workflow_files.is_empty());
}
