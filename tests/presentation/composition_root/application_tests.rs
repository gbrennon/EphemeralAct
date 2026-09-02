#[cfg(test)]
mod tests {
    use ephact::presentation::composition_root::{Application, CompositionRoot};

    use crate::common::fakes::{
        fake_list_actions_port::FakeListActionsPort,
        fake_list_workflows_port::FakeListWorkflowsPort, fake_run_act_port::FakeRunActPort,
    };

    fn compose_application() -> Application {
        CompositionRoot::compose(
            Box::new(FakeRunActPort::new(true)),
            Box::new(FakeListWorkflowsPort::new()),
            Box::new(FakeListActionsPort::new()),
        )
    }

    #[test]
    fn composed_application_runs_help_through_cli_field() {
        let app = compose_application();

        let result = app.cli.run(["ephact"]);

        assert!(result.is_ok());
    }

    #[test]
    fn composed_application_dispatches_list_actions_through_injected_fakes() {
        let app = compose_application();

        let result = app.cli.run(["ephact", "list-actions"]);

        assert!(result.is_ok());
    }
}
