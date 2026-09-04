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
    fn new_creates_cli_instance() {
        let _cli = make_cli();
    }

    #[test]
    fn run_no_args_displays_help() {
        let cli = make_cli();
        let result = cli.run(["ephact"]);
        assert!(result.is_ok());
    }

    #[test]
    fn run_run_subcommand_succeeds() {
        let cli = make_cli();
        let result = cli.run(["ephact", "run"]);
        assert!(result.is_ok());
    }

    #[test]
    fn run_run_subcommand_propagates_workflow_failure() {
        let cli = Cli::new(
            Box::new(FakeRunWorkflowPort::new(false)),
            Box::new(FakeRunAllWorkflowsPort::new(false)),
            Box::new(FakeListWorkflowsPort::new()),
            Box::new(FakeListActionsPort::new()),
        );
        let result = cli.run(["ephact", "run"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("workflow failed"));
    }

    #[test]
    fn run_invalid_subcommand_returns_error() {
        let cli = make_cli();
        let result = cli.run(["ephact", "nonexistent"]);
        assert!(result.is_err());
    }

    #[test]
    fn run_invalid_flag_returns_error() {
        let cli = make_cli();
        let result = cli.run(["ephact", "--nonexistent-flag"]);
        assert!(result.is_err());
    }
}
