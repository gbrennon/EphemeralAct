use serde_json::{Map, Value};

use crate::{
    application::{
        constants::CONTAINER_WORKSPACE,
        dtos::{BuildRunContextRequest, BuildRunContextResponse},
        ports::inbound::build_run_context_port::BuildRunContextPort,
    },
    domain::expression::EvalContext,
};

/// Service that builds the expression context a run's steps are evaluated
/// against, populating the `secrets`, `inputs`, `github`, and `runner`
/// contexts from the run configuration.
pub struct BuildRunContextService;

impl BuildRunContextService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BuildRunContextService {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildRunContextPort for BuildRunContextService {
    fn execute(&self, request: BuildRunContextRequest<'_>) -> BuildRunContextResponse {
        let secrets: Map<String, Value> = request
            .config
            .secrets()
            .iter()
            .map(|secret| {
                (
                    secret.name().to_string(),
                    Value::String(secret.value().into()),
                )
            })
            .collect();
        let inputs: Map<String, Value> = request
            .config
            .inputs()
            .iter()
            .map(|input| (input.key().to_string(), Value::String(input.value().into())))
            .collect();
        let event_name = request
            .config
            .event()
            .map_or("workflow_dispatch", |event| event.as_str());

        let mut event = Map::new();
        event.insert("inputs".into(), Value::Object(inputs.clone()));

        let mut github = Map::new();
        github.insert("event_name".into(), Value::String(event_name.into()));
        github.insert(
            "repository".into(),
            Value::String(request.repository.name().as_str().into()),
        );
        github.insert(
            "workspace".into(),
            Value::String(CONTAINER_WORKSPACE.into()),
        );
        github.insert("event".into(), Value::Object(event));

        let mut runner = Map::new();
        runner.insert("os".into(), Value::String("Linux".into()));
        runner.insert("arch".into(), Value::String("X64".into()));
        runner.insert("temp".into(), Value::String("/tmp".into()));

        let mut context = EvalContext::new();
        context.secrets = Value::Object(secrets);
        context.inputs = Value::Object(inputs);
        context.github = Value::Object(github);
        context.runner = Value::Object(runner);
        BuildRunContextResponse { context }
    }
}
