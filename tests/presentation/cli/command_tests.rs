#[cfg(test)]
mod tests {
    use ephemeral_act::presentation::cli::Cli;

    use crate::common::fakes::{
        fake_list_actions_port::FakeListActionsPort, fake_list_workflows_port::FakeListWorkflowsPort,
        fake_run_act_port::FakeRunActPort,
    };

    fn make_cli() -> Cli {
        Cli::new(
            Box::new(FakeRunActPort::new(true)),
            Box::new(FakeListWorkflowsPort::new()),
            Box::new(FakeListActionsPort::new()),
        )
    }

    #[test]
    fn run_subcommand_dispatches_to_run_handler() {
        let cli = make_cli();

        let result = cli.run(["ephemeral-act", "run"]);

        assert!(result.is_ok());
    }

    #[test]
    fn list_workflows_subcommand_dispatches_to_list_workflows_handler() {
        let cli = make_cli();

        let result = cli.run(["ephemeral-act", "list-workflows"]);

        assert!(result.is_ok());
    }

    #[test]
    fn list_actions_subcommand_dispatches_to_list_actions_handler() {
        let cli = make_cli();

        let result = cli.run(["ephemeral-act", "list-actions"]);

        assert!(result.is_ok());
    }
}
