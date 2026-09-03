use std::collections::HashMap;

use ephact::{
    application::{
        dtos::BuildStepContextRequest,
        ports::outbound::build_step_context_port::BuildStepContextPort,
        services::build_step_context_service::BuildStepContextService,
    },
    domain::expression::EvalContext,
};
use serde_json::Value;

#[test]
fn execute_mirrors_the_environment_into_the_env_context() {
    let mut env = HashMap::new();
    env.insert("MODE".to_string(), "staging".to_string());

    let context = BuildStepContextService::new().execute(BuildStepContextRequest {
        context: &EvalContext::new(),
        env: &env,
    });

    assert_eq!(context.env["MODE"], "staging");
}

#[test]
fn execute_carries_every_other_context_field_over_unchanged() {
    let mut source = EvalContext::new();
    source.secrets = Value::String("secrets".into());
    source.github = Value::String("github".into());
    source.runner = Value::String("runner".into());
    source.inputs = Value::String("inputs".into());

    let context = BuildStepContextService::new().execute(BuildStepContextRequest {
        context: &source,
        env: &HashMap::new(),
    });

    assert_eq!(context.secrets, source.secrets);
    assert_eq!(context.github, source.github);
    assert_eq!(context.runner, source.runner);
    assert_eq!(context.inputs, source.inputs);
}
