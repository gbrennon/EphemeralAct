#[cfg(test)]
mod tests {
    use ephact::{
        application::ports::outbound::event_publisher::EventPublisherPort,
        domain::events::{
            ActRunCompletedPayload, ActionExecutionRequestedPayload, DomainEvent, EventOutcome,
        },
        infrastructure::InMemoryEventBus,
    };

    use crate::common::{
        fakes::{spy_action_handler::SpyActionHandler, spy_cleanup_handler::SpyCleanupHandler},
        fixtures::action_request_fixture::ActionRequestFixture,
    };

    #[test]
    fn publish_act_run_completed_dispatches_to_cleanup_handler() {
        let cleanup_handler = SpyCleanupHandler::new();
        let bus = InMemoryEventBus::new(
            Box::new(cleanup_handler.clone()),
            Box::new(SpyActionHandler::new()),
        );
        let names = vec!["container-a".to_string(), "container-b".to_string()];

        let outcomes = bus.publish(DomainEvent::ActRunCompleted(ActRunCompletedPayload {
            container_names: names.clone(),
            success: true,
        }));

        assert!(outcomes.is_empty());
        assert_eq!(cleanup_handler.cleaned_up(), vec![names]);
    }

    #[test]
    fn publish_act_run_completed_with_empty_names() {
        let cleanup_handler = SpyCleanupHandler::new();
        let bus = InMemoryEventBus::new(
            Box::new(cleanup_handler.clone()),
            Box::new(SpyActionHandler::new()),
        );

        bus.publish(DomainEvent::ActRunCompleted(ActRunCompletedPayload {
            container_names: vec![],
            success: false,
        }));

        assert_eq!(cleanup_handler.cleaned_up(), vec![Vec::<String>::new()]);
    }

    #[test]
    fn publish_action_execution_requested_dispatches_to_action_handler() {
        let action_handler = SpyActionHandler::new();
        let bus = InMemoryEventBus::new(
            Box::new(SpyCleanupHandler::new()),
            Box::new(action_handler.clone()),
        );

        bus.publish(DomainEvent::ActionExecutionRequested(Box::new(
            ActionExecutionRequestedPayload {
                request: ActionRequestFixture::for_action(
                    "https://data.forgejo.org/actions/cache@v4",
                ),
            },
        )));

        assert_eq!(
            action_handler.requested(),
            vec!["https://data.forgejo.org/actions/cache@v4".to_string()]
        );
    }

    #[test]
    fn publish_action_execution_requested_returns_the_handler_outcome() {
        let bus = InMemoryEventBus::new(
            Box::new(SpyCleanupHandler::new()),
            Box::new(SpyActionHandler::new()),
        );

        let outcomes = bus.publish(DomainEvent::ActionExecutionRequested(Box::new(
            ActionExecutionRequestedPayload {
                request: ActionRequestFixture::for_action("./actions/publish"),
            },
        )));

        assert_eq!(outcomes.len(), 1);
        let EventOutcome::ActionExecuted(result) = &outcomes[0];
        assert_eq!(result.as_ref().unwrap().stdout, "action ran\n");
    }

    #[test]
    fn publish_action_execution_requested_does_not_clean_up_containers() {
        let cleanup_handler = SpyCleanupHandler::new();
        let bus = InMemoryEventBus::new(
            Box::new(cleanup_handler.clone()),
            Box::new(SpyActionHandler::new()),
        );

        bus.publish(DomainEvent::ActionExecutionRequested(Box::new(
            ActionExecutionRequestedPayload {
                request: ActionRequestFixture::for_action("./actions/publish"),
            },
        )));

        assert!(cleanup_handler.cleaned_up().is_empty());
    }
}
