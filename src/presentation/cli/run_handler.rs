use super::run_args::RunArgs;
use crate::{
    application::{
        dtos::{RunAllWorkflowsRequest, RunSummary, RunWorkflowRequest},
        ports::inbound::{
            run_all_workflows_port::RunAllWorkflowsPort, run_workflow_port::RunWorkflowPort,
        },
    },
    domain::workflow::StepType,
};

/// Handles the `run` subcommand by dispatching parsed CLI arguments to the
/// application port.
///
/// Owns the presentation concerns (summary rendering, step output relay,
/// result interpretation) so that neither the domain core nor the CLI
/// argument parser needs to know about terminal I/O or exit semantics.
pub struct RunHandler;

impl RunHandler {
    /// Executes the `run` subcommand: converts CLI args to domain objects,
    pub fn handle(
        args: RunArgs,
        run_workflow_port: &dyn RunWorkflowPort,
        run_all_workflows_port: &dyn RunAllWorkflowsPort,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let (config, repository) = args.to_domain()?;
        let summary = if config.all_workflows() {
            run_all_workflows_port.execute(RunAllWorkflowsRequest::new(config, repository))?
        } else {
            run_workflow_port.execute(RunWorkflowRequest::new(config, repository))?
        };
        eprint!("{}", Self::render(&summary));
        if !summary.success {
            return Err("workflow failed".into());
        }
        Ok(())
    }
    /// Renders the run summary as plain text for the terminal: a workflow
    /// header, per-job and per-step status lines, and each step's raw output.
    pub fn render(summary: &RunSummary) -> String {
        let mut out = String::new();
        let status = if summary.success {
            "succeeded"
        } else {
            "failed"
        };
        out.push_str(&format!(
            "Workflow '{}': {} ({:?})\n",
            summary.name, status, summary.duration
        ));
        for job in &summary.job_summaries {
            let job_status = if job.success { "succeeded" } else { "failed" };
            let job_label = job.name.as_deref().unwrap_or(&job.job_id);
            out.push_str(&format!("  Job '{}': {}\n", job_label, job_status));
            for step in &job.steps {
                let kind = match step.step_type {
                    StepType::Run => "run",
                    StepType::Uses => "uses",
                    StepType::Composite => "composite",
                    StepType::Invalid => "invalid",
                };
                let outcome = match step.exit_code {
                    Some(0) => "ok".to_string(),
                    Some(code) => format!("failed (exit code: {})", code),
                    None => "error".to_string(),
                };
                out.push_str(&format!(
                    "    Step '{}' ({}): {}\n",
                    step.name, kind, outcome
                ));
                if step.continue_on_error {
                    out.push_str("      (continue-on-error)\n");
                }
                out.push_str(&step.stdout);
                out.push_str(&step.stderr);
            }
        }
        out
    }
}
