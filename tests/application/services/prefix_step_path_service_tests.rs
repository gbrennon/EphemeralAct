use std::collections::HashMap;

use ephact::application::{
    dtos::PrefixStepPathRequest, ports::inbound::prefix_step_path_port::PrefixStepPathPort,
    services::prefix_step_path_service::PrefixStepPathService,
};

fn env_with_path(path: &str) -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("PATH".to_string(), path.to_string());
    env
}

#[test]
fn execute_leaves_the_path_untouched_without_additions() {
    let env = env_with_path("/usr/bin");

    let result = PrefixStepPathService::new().execute(PrefixStepPathRequest {
        env: &env,
        path_additions: &[],
    });

    assert_eq!(result.get("PATH").map(String::as_str), Some("/usr/bin"));
}

#[test]
fn execute_prefixes_the_additions_before_the_existing_path() {
    let env = env_with_path("/usr/bin");

    let result = PrefixStepPathService::new().execute(PrefixStepPathRequest {
        env: &env,
        path_additions: &["/opt/bin".to_string(), "/opt/tools".to_string()],
    });

    assert_eq!(
        result.get("PATH").map(String::as_str),
        Some("/opt/bin:/opt/tools:/usr/bin")
    );
}

#[test]
fn execute_appends_an_empty_segment_when_the_environment_has_no_path() {
    let result = PrefixStepPathService::new().execute(PrefixStepPathRequest {
        env: &HashMap::new(),
        path_additions: &["/opt/bin".to_string()],
    });

    assert_eq!(result.get("PATH").map(String::as_str), Some("/opt/bin:"));
}

#[test]
fn execute_preserves_the_other_environment_entries() {
    let mut env = env_with_path("/usr/bin");
    env.insert("MODE".to_string(), "staging".to_string());

    let result = PrefixStepPathService::new().execute(PrefixStepPathRequest {
        env: &env,
        path_additions: &["/opt/bin".to_string()],
    });

    assert_eq!(result.get("MODE").map(String::as_str), Some("staging"));
}
