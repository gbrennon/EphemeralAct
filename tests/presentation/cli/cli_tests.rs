#[cfg(test)]
mod tests {
    use ephemeral_act::presentation::cli::Cli;

    use crate::common::fakes::{
        fake_list_actions_port::FakeListActionsPort,
        fake_list_workflows_port::FakeListWorkflowsPort, fake_run_act_port::FakeRunActPort,
    };

    fn make_cli() -> Cli {
        Cli::new(
            Box::new(FakeRunActPort::new(true)),
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
        let result = cli.run(["ephemeral-act"]);
        assert!(result.is_ok());
    }

    #[test]
    fn run_run_subcommand_succeeds() {
        let cli = make_cli();
        let result = cli.run(["ephemeral-act", "run"]);
        assert!(result.is_ok());
    }

    #[test]
    fn run_run_subcommand_propagates_workflow_failure() {
        let cli = Cli::new(
            Box::new(FakeRunActPort::new(false)),
            Box::new(FakeListWorkflowsPort::new()),
            Box::new(FakeListActionsPort::new()),
        );
        let result = cli.run(["ephemeral-act", "run"]);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("workflow failed"));
    }

    #[test]
    fn run_invalid_subcommand_returns_error() {
        let cli = make_cli();
        let result = cli.run(["ephemeral-act", "nonexistent"]);
        assert!(result.is_err());
    }

    #[test]
    fn run_invalid_flag_returns_error() {
        let cli = make_cli();
        let result = cli.run(["ephemeral-act", "--nonexistent-flag"]);
        assert!(result.is_err());
    }
}
