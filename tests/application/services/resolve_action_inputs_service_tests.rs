use ephact::{
    application::{
        dtos::ResolveActionInputsRequest,
        ports::outbound::resolve_action_inputs_port::ResolveActionInputsPort,
        services::resolve_action_inputs_service::ResolveActionInputsService,
    },
    domain::workflow::{ActionDefinition, Step},
};

const WITH_DEFAULTS: &str = "name: Deploy\ninputs:\n  mode:\n    description: target\n    default: production\n  token:\n    description: secret\nruns:\n  using: composite\n  steps: []\n";

fn definition() -> ActionDefinition {
    serde_yaml::from_str(WITH_DEFAULTS).unwrap()
}

fn step(yaml: &str) -> Step {
    serde_yaml::from_str(yaml).unwrap()
}

#[test]
fn execute_returns_the_declared_defaults() {
    let inputs = ResolveActionInputsService::new().execute(ResolveActionInputsRequest {
        definition: &definition(),
        step: &step("uses: ./actions/deploy\n"),
    });

    assert_eq!(inputs.get("mode").map(String::as_str), Some("production"));
}

#[test]
fn execute_lets_with_override_a_default() {
    let inputs = ResolveActionInputsService::new().execute(ResolveActionInputsRequest {
        definition: &definition(),
        step: &step("uses: ./actions/deploy\nwith:\n  mode: staging\n"),
    });

    assert_eq!(inputs.get("mode").map(String::as_str), Some("staging"));
}

#[test]
fn execute_omits_an_input_with_neither_default_nor_with() {
    let inputs = ResolveActionInputsService::new().execute(ResolveActionInputsRequest {
        definition: &definition(),
        step: &step("uses: ./actions/deploy\n"),
    });

    assert!(!inputs.contains_key("token"));
}

#[test]
fn execute_passes_undeclared_with_keys_through() {
    let inputs = ResolveActionInputsService::new().execute(ResolveActionInputsRequest {
        definition: &definition(),
        step: &step("uses: ./actions/deploy\nwith:\n  extra: value\n"),
    });

    assert_eq!(inputs.get("extra").map(String::as_str), Some("value"));
}
