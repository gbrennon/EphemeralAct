#[cfg(test)]
mod tests {
    use ephact::presentation::cli::{ListActionsHandler, parse_list_actions_test_args};

    use crate::common::fakes::fake_list_actions_port::FakeListActionsPort;

    #[test]
    fn handle_with_fake_port_succeeds_on_empty_response() {
        let args = parse_list_actions_test_args(&[]);
        let port = FakeListActionsPort::new();

        let result = ListActionsHandler::handle(args, &port);

        assert!(result.is_ok());
    }
}
