#[cfg(test)]
mod tests {
    use ephact::presentation::cli::Cli;

    use crate::common::fakes::{
        fake_list_actions_port::FakeListActionsPort,
        fake_list_workflows_port::FakeListWorkflowsPort,
        fake_run_all_workflows_port::FakeRunAllWorkflowsPort,
        fake_run_workflow_port::FakeRunWorkflowPort,
    };

    fn make_cli() -> Cli {
        Cli::new(
            Box::new(FakeRunWorkflowPort::new(true)),
            Box::new(FakeRunAllWorkflowsPort::new(true)),
            Box::new(FakeListWorkflowsPort::new()),
            Box::new(FakeListActionsPort::new()),
        )
    }

    #[test]
    fn run_subcommand_dispatches_to_run_handler() {
        let cli = make_cli();

        let result = cli.run(["ephact", "run"]);

        assert!(result.is_ok());
    }

    #[test]
    fn list_workflows_subcommand_dispatches_to_list_workflows_handler() {
        let cli = make_cli();

        let result = cli.run(["ephact", "list-workflows"]);

        assert!(result.is_ok());
    }

    #[test]
    fn list_actions_subcommand_dispatches_to_list_actions_handler() {
        let cli = make_cli();

        let result = cli.run(["ephact", "list-actions"]);

        assert!(result.is_ok());
    }
}
