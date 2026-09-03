use std::fs;

use ephact::application::{
    dtos::LoadWorkflowRequest, ports::outbound::load_workflow_port::LoadWorkflowPort,
    services::load_workflow_service::LoadWorkflowService,
};

#[test]
fn execute_parses_a_valid_workflow_file() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("ci.yml");
    fs::write(
        &path,
        "name: Ci\non: push\nenv:\n  MODE: staging\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hi\n",
    )
    .unwrap();

    let workflow = LoadWorkflowService::new()
        .execute(LoadWorkflowRequest {
            workflow_file: &path,
        })
        .unwrap();

    assert_eq!(workflow.name.as_deref(), Some("Ci"));
    assert_eq!(
        workflow.env.get("MODE").map(String::as_str),
        Some("staging")
    );
    assert!(workflow.jobs.contains_key("build"));
}

#[test]
fn execute_errors_for_a_missing_file() {
    let tmp = tempfile::tempdir().unwrap();

    let result = LoadWorkflowService::new().execute(LoadWorkflowRequest {
        workflow_file: &tmp.path().join("absent.yml"),
    });

    assert!(result.is_err());
}

#[test]
fn execute_errors_for_malformed_yaml() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("broken.yml");
    fs::write(&path, "name: [unterminated\n").unwrap();

    let result = LoadWorkflowService::new().execute(LoadWorkflowRequest {
        workflow_file: &path,
    });

    assert!(result.is_err());
}
