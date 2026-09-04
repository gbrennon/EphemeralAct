use std::{collections::HashMap, path::PathBuf, sync::Arc};

use ephact::{
    application::{
        dtos::{ExecuteActionResponse, RunActionRequest},
        ports::inbound::RunActionPort,
        services::run_action_service::RunActionService,
    },
    domain::{expression::EvalContext, workflow::Step},
};

use crate::common::fakes::{fake_command_bus::FakeCommandBus, stub_container::StubContainer};

#[test]
fn execute_delegates_action_execution_to_command_bus() {
    let command_bus = Arc::new(
        FakeCommandBus::new().with_action_result(ExecuteActionResponse {
            exit_code: 0,
            stdout: "action executed".into(),
            stderr: String::new(),
        }),
    );

    let service = RunActionService::new(command_bus.clone());
    let step: Step = serde_yaml::from_str("uses: actions/checkout@v4").unwrap();

    let request = RunActionRequest::new(
        "actions/checkout@v4".into(),
        step,
        PathBuf::from("/repo"),
        HashMap::new(),
        EvalContext::new(),
        Arc::new(StubContainer),
    );

    let response = service.execute(request).unwrap();

    assert_eq!(response.exit_code, 0);
    assert_eq!(response.stdout, "action executed");
    assert_eq!(command_bus.dispatched_actions.lock().len(), 1);
}
