use ephact::{
    application::ports::outbound::load_action_definition_port::LoadActionDefinitionPort,
    infrastructure::actions::load_action_definition_service::LoadActionDefinitionService,
};
use std::fs;

use ephact::{application::dtos::LoadActionDefinitionRequest, domain::workflow::ActionRuns};

const COMPOSITE: &str =
    "name: Greet\nruns:\n  using: composite\n  steps:\n    - run: echo hi\n      shell: bash\n";

#[test]
fn execute_loads_action_yml() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("action.yml"), COMPOSITE).unwrap();

    let definition = LoadActionDefinitionService::new()
        .execute(LoadActionDefinitionRequest {
            action_dir: tmp.path(),
        })
        .unwrap();

    assert_eq!(definition.name, "Greet");
    assert!(matches!(definition.runs, ActionRuns::Composite { .. }));
}

#[test]
fn execute_falls_back_to_action_yaml() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("action.yaml"), COMPOSITE).unwrap();

    let definition = LoadActionDefinitionService::new()
        .execute(LoadActionDefinitionRequest {
            action_dir: tmp.path(),
        })
        .unwrap();

    assert_eq!(definition.name, "Greet");
}

#[test]
fn execute_errors_when_no_definition_is_present() {
    let tmp = tempfile::tempdir().unwrap();

    let error = LoadActionDefinitionService::new()
        .execute(LoadActionDefinitionRequest {
            action_dir: tmp.path(),
        })
        .unwrap_err();

    assert_eq!(
        error.message,
        format!("action.yml not found in {}", tmp.path().display())
    );
}

#[test]
fn execute_errors_on_malformed_yaml() {
    let tmp = tempfile::tempdir().unwrap();
    fs::write(tmp.path().join("action.yml"), "name: [unterminated\n").unwrap();

    let error = LoadActionDefinitionService::new()
        .execute(LoadActionDefinitionRequest {
            action_dir: tmp.path(),
        })
        .unwrap_err();

    assert!(
        error.message.starts_with("failed to parse "),
        "{}",
        error.message
    );
}
