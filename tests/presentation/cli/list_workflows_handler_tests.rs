#[cfg(test)]
mod tests {
    use ephemeral_act::presentation::cli::{ListWorkflowsHandler, parse_list_workflows_test_args};

    use crate::common::fakes::fake_list_workflows_port::FakeListWorkflowsPort;

    #[test]
    fn handle_with_fake_port_succeeds_on_empty_response() {
        let args = parse_list_workflows_test_args(&[]);
        let port = FakeListWorkflowsPort::new();

        let result = ListWorkflowsHandler::handle(args, &port);

        assert!(result.is_ok());
    }
}
