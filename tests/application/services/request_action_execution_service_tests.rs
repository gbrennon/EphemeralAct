use std::{collections::HashMap, path::PathBuf, rc::Rc, sync::Arc};

use ephact::{
    application::{
        dtos::ExecuteActionRequest,
        ports::outbound::request_action_execution_port::RequestActionExecutionPort,
        services::request_action_execution_service::RequestActionExecutionService,
    },
    domain::{events::DomainEvent, expression::EvalContext},
};

use crate::common::fakes::{
    fake_event_publisher::FakeEventPublisher, spy_action_handler::SpyActionHandler,
    stub_container::StubContainer,
};

fn request(action_ref: &str) -> ExecuteActionRequest {
    ExecuteActionRequest {
        action_ref: action_ref.to_string(),
        step: serde_yaml::from_str("uses: ./actions/greet\n").unwrap(),
        repo_path: PathBuf::from("/repo"),
        env: HashMap::new(),
        context: EvalContext::new(),
        container: Arc::new(StubContainer),
    }
}

#[test]
fn execute_publishes_an_action_execution_requested_event_and_returns_the_outcome() {
    let handler = SpyActionHandler::new();
    let publisher = FakeEventPublisher::with_action_handler(Rc::new(handler.clone()));
    let service = RequestActionExecutionService::new(Arc::new(publisher.clone()));

    let response = service.execute(request("./actions/greet")).unwrap();

    assert_eq!(response.stdout, "action ran\n");
    assert_eq!(handler.requested(), vec!["./actions/greet".to_string()]);
    let events = publisher.events();
    let Some(DomainEvent::ActionExecutionRequested(payload)) = events.first() else {
        panic!("expected an action execution request, got {events:?}");
    };
    assert_eq!(payload.request.action_ref, "./actions/greet");
}

#[test]
fn execute_fails_the_step_when_no_handler_runs_the_action() {
    let service = RequestActionExecutionService::new(Arc::new(FakeEventPublisher::new()));

    let error = service.execute(request("./actions/greet")).unwrap_err();

    assert_eq!(
        error.message,
        "no handler executed the action './actions/greet'"
    );
}
