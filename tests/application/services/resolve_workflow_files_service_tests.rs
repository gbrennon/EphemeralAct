use std::path::{Path, PathBuf};

use ephact::{
    application::{
        dtos::ResolveWorkflowFilesRequest,
        ports::inbound::resolve_workflow_files_port::ResolveWorkflowFilesPort,
        services::resolve_workflow_files_service::ResolveWorkflowFilesService,
    },
    domain::{ActRunConfig, ActWorkflow},
};

use crate::common::fakes::{
    fake_detect_workflow_file_port::FakeDetectWorkflowFilePort,
    fake_list_all_workflow_files_port::FakeListAllWorkflowFilesPort,
    fake_resolve_named_workflow_file_port::FakeResolveNamedWorkflowFilePort,
};

#[test]
fn execute_lists_every_workflow_when_all_workflows_is_configured() {
    let all_lister = FakeListAllWorkflowFilesPort::returning(vec![
        PathBuf::from("a.yml"),
        PathBuf::from("b.yml"),
    ]);
    let detector = FakeDetectWorkflowFilePort::returning(PathBuf::from("detected.yml"));
    let service = ResolveWorkflowFilesService::new(
        Box::new(all_lister),
        Box::new(FakeResolveNamedWorkflowFilePort::returning(PathBuf::from(
            "named.yml",
        ))),
        Box::new(detector),
    );
    let config = ActRunConfig::new().with_all_workflows(true);

    let response = service
        .execute(ResolveWorkflowFilesRequest {
            config: &config,
            repo_path: Path::new("/repo"),
        })
        .unwrap();

    assert_eq!(
        response.workflow_files,
        vec![PathBuf::from("a.yml"), PathBuf::from("b.yml")]
    );
}

#[test]
fn execute_resolves_the_configured_workflow_by_name() {
    let named_resolver = FakeResolveNamedWorkflowFilePort::returning(PathBuf::from("named.yml"));
    let service = ResolveWorkflowFilesService::new(
        Box::new(FakeListAllWorkflowFilesPort::returning(vec![])),
        Box::new(named_resolver),
        Box::new(FakeDetectWorkflowFilePort::returning(PathBuf::from(
            "detected.yml",
        ))),
    );
    let config = ActRunConfig::new().with_workflow(ActWorkflow::new("ci.yml".into()));

    let response = service
        .execute(ResolveWorkflowFilesRequest {
            config: &config,
            repo_path: Path::new("/repo"),
        })
        .unwrap();

    assert_eq!(response.workflow_files, vec![PathBuf::from("named.yml")]);
}

#[test]
fn execute_detects_the_workflow_when_none_is_configured() {
    let service = ResolveWorkflowFilesService::new(
        Box::new(FakeListAllWorkflowFilesPort::returning(vec![])),
        Box::new(FakeResolveNamedWorkflowFilePort::returning(PathBuf::from(
            "named.yml",
        ))),
        Box::new(FakeDetectWorkflowFilePort::returning(PathBuf::from(
            "detected.yml",
        ))),
    );
    let config = ActRunConfig::new();

    let response = service
        .execute(ResolveWorkflowFilesRequest {
            config: &config,
            repo_path: Path::new("/repo"),
        })
        .unwrap();

    assert_eq!(response.workflow_files, vec![PathBuf::from("detected.yml")]);
}

#[test]
fn execute_does_not_consult_the_detector_for_a_named_workflow() {
    let detector = FakeDetectWorkflowFilePort::returning(PathBuf::from("detected.yml"));
    let config = ActRunConfig::new().with_workflow(ActWorkflow::new("ci.yml".into()));
    let service = ResolveWorkflowFilesService::new(
        Box::new(FakeListAllWorkflowFilesPort::returning(vec![])),
        Box::new(FakeResolveNamedWorkflowFilePort::returning(PathBuf::from(
            "named.yml",
        ))),
        Box::new(detector),
    );

    service
        .execute(ResolveWorkflowFilesRequest {
            config: &config,
            repo_path: Path::new("/repo"),
        })
        .unwrap();
}

#[test]
fn execute_propagates_a_collaborator_error() {
    let service = ResolveWorkflowFilesService::new(
        Box::new(FakeListAllWorkflowFilesPort::returning(vec![])),
        Box::new(FakeResolveNamedWorkflowFilePort::failing(
            "workflow file not found: ci.yml",
        )),
        Box::new(FakeDetectWorkflowFilePort::returning(PathBuf::from(
            "detected.yml",
        ))),
    );
    let config = ActRunConfig::new().with_workflow(ActWorkflow::new("ci.yml".into()));

    let error = service
        .execute(ResolveWorkflowFilesRequest {
            config: &config,
            repo_path: Path::new("/repo"),
        })
        .unwrap_err()
        .to_string();

    assert_eq!(error, "workflow file not found: ci.yml");
}
