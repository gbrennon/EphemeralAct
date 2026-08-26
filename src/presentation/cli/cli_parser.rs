use clap::Parser;

/// CLI argument parser backed by clap.
///
/// This struct is only used internally by [`Cli`] at parse time; consumers
/// never interact with it directly.
#[derive(Parser)]
#[command(
    name = "ephemeral-act",
    about = "Run GitHub Actions locally in ephemeral repositories",
    long_about = "Runs CI workflows in an ephemeral copy of a repository using \
                  `act`. The CI host is auto-detected from the \
                  repository layout; see `run --help` for the available options.",
    arg_required_else_help = true,
    after_long_help = r#"EXAMPLES:
    ephemeral-act run
    ephemeral-act run --workflow ci.yml --job test
    ephemeral-act run --event push --secret TOKEN=abc123
    ephemeral-act run --container-engine docker

CI host from the repository layout and manages ephemeral copies internally."#
)]
pub struct CliParser {
    #[command(subcommand)]
    pub(crate) command: super::command::Command,
}

/// Parses CLI arguments for the `run` subcommand from a string slice.
///
/// Intended for use in tests — avoids depending on `std::env::args()`.
pub fn parse_run_test_args(args: &[&str]) -> super::run_args::RunArgs {
    let mut full: Vec<&str> = vec!["ephemeral-act", "run"];
    full.extend_from_slice(args);
    let cli = CliParser::parse_from(&full);
    match cli.command {
        super::command::Command::Run(args) => *args,
        _ => unreachable!(),
    }
}

/// Parses CLI arguments for the `list-workflows` subcommand from a string slice.
///
/// Intended for use in tests.
pub fn parse_list_workflows_test_args(args: &[&str]) -> super::list_workflows_args::ListWorkflowsArgs {
    let mut full: Vec<&str> = vec!["ephemeral-act", "list-workflows"];
    full.extend_from_slice(args);
    let cli = CliParser::parse_from(&full);
    match cli.command {
        super::command::Command::ListWorkflows(args) => *args,
        _ => unreachable!(),
    }
}

/// Parses CLI arguments for the `list-actions` subcommand from a string slice.
///
/// Intended for use in tests.
pub fn parse_list_actions_test_args(args: &[&str]) -> super::list_actions_args::ListActionsArgs {
    let mut full: Vec<&str> = vec!["ephemeral-act", "list-actions"];
    full.extend_from_slice(args);
    let cli = CliParser::parse_from(&full);
    match cli.command {
        super::command::Command::ListActions(args) => *args,
        _ => unreachable!(),
    }
}
