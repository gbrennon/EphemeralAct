use ephact::{
    application::ports::outbound::load_workflow_port::LoadWorkflowPort,
    infrastructure::workflows::load_workflow_service::LoadWorkflowService,
};

use ephact::application::dtos::LoadWorkflowRequest;

const VALID_WORKFLOW: &str = "name: Ci\non: push\nenv:\n  MODE: staging\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hi\n";

#[test]
fn execute_parses_valid_workflow_content() {
    let workflow = LoadWorkflowService::new()
        .execute(LoadWorkflowRequest {
            workflow_content: VALID_WORKFLOW,
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
fn execute_errors_for_content_that_is_not_a_workflow_document() {
    let result = LoadWorkflowService::new().execute(LoadWorkflowRequest {
        workflow_content: "- push\n- pull_request\n",
    });

    assert!(result.is_err());
}

#[test]
fn execute_errors_for_malformed_yaml() {
    let result = LoadWorkflowService::new().execute(LoadWorkflowRequest {
        workflow_content: "name: [unterminated\n",
    });

    assert!(result.is_err());
}
