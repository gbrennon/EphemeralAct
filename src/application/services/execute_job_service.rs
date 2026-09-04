use std::{error::Error, sync::Arc, time::Instant};

use crate::application::commands::ExecuteStepCommand;
use crate::application::dtos::BuildJobEnvironmentRequest;
use crate::application::dtos::BuildStepContextRequest;
use crate::application::dtos::ExecuteJobRequest;
use crate::application::dtos::JobExecution;
use crate::application::dtos::JobSummary;
use crate::application::dtos::PrefixStepPathRequest;
use crate::application::dtos::PrepareJobContainerRequest;
use crate::application::dtos::ReadStepExportsRequest;
use crate::application::dtos::StepSummary;
use crate::application::dtos::SummarizeStepRequest;
use crate::application::ports::inbound::execute_job_port::ExecuteJobPort;
use crate::application::ports::outbound::build_job_environment_port::BuildJobEnvironmentPort;
use crate::application::ports::outbound::build_step_context_port::BuildStepContextPort;
use crate::application::ports::outbound::command_bus_port::CommandBusPort;
use crate::application::ports::outbound::prefix_step_path_port::PrefixStepPathPort;
use crate::application::ports::outbound::prepare_job_container_port::PrepareJobContainerPort;
use crate::application::ports::outbound::read_step_exports_port::ReadStepExportsPort;
use crate::application::ports::outbound::summarize_step_port::SummarizeStepPort;

/// Application service coordinating the execution of one job.
///
/// Builds the job environment and container through outbound ports, then
/// publishes one [`ExecuteStepCommand`] per step: the step command handler
/// runs each step, so this service never depends on the step entrypoint.
pub struct ExecuteJobService {
    job_environment_builder: Box<dyn BuildJobEnvironmentPort>,
    container_preparer: Box<dyn PrepareJobContainerPort>,
    step_path_prefixer: Box<dyn PrefixStepPathPort>,
    step_context_builder: Box<dyn BuildStepContextPort>,
    step_summarizer: Box<dyn SummarizeStepPort>,
    step_exports_reader: Box<dyn ReadStepExportsPort>,
    command_bus: Arc<dyn CommandBusPort>,
}

impl ExecuteJobService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        job_environment_builder: Box<dyn BuildJobEnvironmentPort>,
        container_preparer: Box<dyn PrepareJobContainerPort>,
        step_path_prefixer: Box<dyn PrefixStepPathPort>,
        step_context_builder: Box<dyn BuildStepContextPort>,
        step_summarizer: Box<dyn SummarizeStepPort>,
        step_exports_reader: Box<dyn ReadStepExportsPort>,
        command_bus: Arc<dyn CommandBusPort>,
    ) -> Self {
        Self {
            job_environment_builder,
            container_preparer,
            step_path_prefixer,
            step_context_builder,
            step_summarizer,
            step_exports_reader,
            command_bus,
        }
    }
}

impl ExecuteJobPort for ExecuteJobService {
    fn execute(&self, request: ExecuteJobRequest<'_>) -> Result<JobExecution, Box<dyn Error>> {
        let mut step_env = self
            .job_environment_builder
            .execute(BuildJobEnvironmentRequest {
                workflow: request.workflow,
                job_env: &request.run.job.env,
            })
            .env;

        let prepared = self
            .container_preparer
            .execute(PrepareJobContainerRequest {
                job_id: &request.run.job_id,
                runs_on: request.run.job.runs_on.as_deref(),
                repo_path: request.repo_path,
            })?;

        let mut extra_path: Vec<String> = Vec::new();
        let mut job_success = true;
        let mut steps: Vec<StepSummary> = Vec::new();

        for step in &request.run.job.steps {
            step_env = self.step_path_prefixer.execute(PrefixStepPathRequest {
                env: &step_env,
                path_additions: &extra_path,
            });

            let started_at = Instant::now();
            let step_context = self.step_context_builder.execute(BuildStepContextRequest {
                context: request.context,
                env: &step_env,
            });

            let outcome = self.command_bus.dispatch_step(ExecuteStepCommand::new(
                step.clone(),
                step_env.clone(),
                step_context,
                prepared.container.clone(),
                request.repo_path.to_path_buf(),
            ));

            let summarized = self.step_summarizer.execute(SummarizeStepRequest {
                step,
                outcome,
                duration: started_at.elapsed(),
            });
            job_success &= !summarized.fails_job;
            steps.push(summarized.summary);

            let exports = self.step_exports_reader.execute(ReadStepExportsRequest {
                container: prepared.container.as_ref(),
            });
            extra_path.extend(exports.path_additions);
            step_env.extend(exports.env);
        }

        Ok(JobExecution {
            job_summary: JobSummary {
                job_id: request.run.job_id.clone(),
                name: request.run.job.name.clone(),
                steps,
                success: job_success,
            },
            container_name: prepared.container_name,
        })
    }
}
