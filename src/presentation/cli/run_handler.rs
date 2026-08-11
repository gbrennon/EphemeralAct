use super::run_args::RunArgs;
use crate::core::ports::inbound::run_act_port::RunActUseCase;

/// Handles the `run` subcommand by dispatching parsed CLI arguments to the
/// application use case.
///
/// Owns the presentation concerns (stdout/stderr formatting,
/// result interpretation) so that neither the domain core nor the CLI
/// argument parser needs to know about terminal I/O or exit semantics.
pub struct RunHandler;

impl RunHandler {
    /// Executes the `run` subcommand: converts CLI args to domain objects,
    /// calls the use case, prints output, and returns an error when the
    /// workflow reports failure.
    pub fn handle(
        args: RunArgs,
        use_case: &dyn RunActUseCase,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (config, repository) = args.to_domain()?;
        let result = use_case.run_act(config, repository)?;

        if !result.stdout.is_empty() {
            eprintln!("{}", result.stdout);
        }
        if !result.stderr.is_empty() {
            eprintln!("{}", result.stderr);
        }
        if !result.success {
            return Err("workflow failed".into());
        }
        Ok(())
    }
}
