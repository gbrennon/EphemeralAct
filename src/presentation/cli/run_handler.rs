use super::run_args::RunArgs;
use crate::core::{dtos::RunActRequest, ports::inbound::run_act_port::RunActPort};

/// Handles the `run` subcommand by dispatching parsed CLI arguments to the
/// application use case.
///
/// Owns the presentation concerns (step output relay, result
/// interpretation) so that neither the domain core nor the CLI
/// argument parser needs to know about terminal I/O or exit semantics.
pub struct RunHandler;

impl RunHandler {
    /// Executes the `run` subcommand: converts CLI args to domain objects,
    /// calls the use case, relays step output, and returns an error when the
    /// workflow reports failure.
    pub fn handle(
        args: RunArgs,
        use_case: &dyn RunActPort,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (config, repository) = args.to_domain()?;
        let summary = use_case.execute(RunActRequest::new(config, repository))?;

        for job in &summary.job_summaries {
            for step in &job.steps {
                if !step.stdout.is_empty() {
                    eprintln!("{}", step.stdout);
                }
                if !step.stderr.is_empty() {
                    eprintln!("{}", step.stderr);
                }
            }
        }

        if !summary.success {
            return Err("workflow failed".into());
        }
        Ok(())
    }
}
