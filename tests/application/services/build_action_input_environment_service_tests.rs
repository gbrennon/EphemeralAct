use std::collections::HashMap;

use ephact::application::{
    dtos::BuildActionInputEnvironmentRequest,
    ports::outbound::build_action_input_environment_port::BuildActionInputEnvironmentPort,
    services::build_action_input_environment_service::BuildActionInputEnvironmentService,
};

fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

#[test]
fn execute_sets_the_action_path() {
    let env =
        BuildActionInputEnvironmentService::new().execute(BuildActionInputEnvironmentRequest {
            env: &HashMap::new(),
            inputs: &HashMap::new(),
            action_path: "/tmp/actions/greet",
        });

    assert_eq!(
        env.get("GITHUB_ACTION_PATH").map(String::as_str),
        Some("/tmp/actions/greet")
    );
}

#[test]
fn execute_exposes_inputs_as_upper_snake_case_variables() {
    let env =
        BuildActionInputEnvironmentService::new().execute(BuildActionInputEnvironmentRequest {
            env: &HashMap::new(),
            inputs: &map(&[("my input", "value")]),
            action_path: "/tmp/actions/greet",
        });

    assert_eq!(env.get("INPUT_MY_INPUT").map(String::as_str), Some("value"));
}

#[test]
fn execute_preserves_existing_environment_entries() {
    let env =
        BuildActionInputEnvironmentService::new().execute(BuildActionInputEnvironmentRequest {
            env: &map(&[("MODE", "staging")]),
            inputs: &HashMap::new(),
            action_path: "/tmp/actions/greet",
        });

    assert_eq!(env.get("MODE").map(String::as_str), Some("staging"));
}

#[test]
fn execute_lets_an_input_win_over_a_colliding_environment_entry() {
    let env =
        BuildActionInputEnvironmentService::new().execute(BuildActionInputEnvironmentRequest {
            env: &map(&[("INPUT_MODE", "from-env")]),
            inputs: &map(&[("mode", "from-input")]),
            action_path: "/tmp/actions/greet",
        });

    assert_eq!(
        env.get("INPUT_MODE").map(String::as_str),
        Some("from-input")
    );
}
