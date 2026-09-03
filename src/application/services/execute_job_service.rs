use std::{error::Error, time::Instant};

use crate::application::{
    dtos::{
        BuildJobEnvironmentRequest, BuildStepContextRequest, ExecuteJobRequest, ExecuteStepRequest,
        JobExecution, JobSummary, PrefixStepPathRequest, PrepareJobContainerRequest,
        ReadStepExportsRequest, StepSummary, SummarizeStepRequest,
    },
    ports::{
        inbound::{
            build_step_context_port::BuildStepContextPort, execute_job_port::ExecuteJobPort,
            execute_step_port::ExecuteStepPort, prefix_step_path_port::PrefixStepPathPort,
            prepare_job_container_port::PrepareJobContainerPort,
            read_step_exports_port::ReadStepExportsPort, summarize_step_port::SummarizeStepPort,
        },
        outbound::build_job_environment_port::BuildJobEnvironmentPort,
    },
};

/// Service that runs one planned job inside a fresh ephemeral container,
/// carrying each step's exports over to the steps that follow it.
pub struct ExecuteJobService {
    job_environment_builder: Box<dyn BuildJobEnvironmentPort>,
    container_preparer: Box<dyn PrepareJobContainerPort>,
    step_path_prefixer: Box<dyn PrefixStepPathPort>,
    step_context_builder: Box<dyn BuildStepContextPort>,
    step_executor: Box<dyn ExecuteStepPort>,
    step_summarizer: Box<dyn SummarizeStepPort>,
    step_exports_reader: Box<dyn ReadStepExportsPort>,
}

impl ExecuteJobService {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        job_environment_builder: Box<dyn BuildJobEnvironmentPort>,
        container_preparer: Box<dyn PrepareJobContainerPort>,
        step_path_prefixer: Box<dyn PrefixStepPathPort>,
        step_context_builder: Box<dyn BuildStepContextPort>,
        step_executor: Box<dyn ExecuteStepPort>,
        step_summarizer: Box<dyn SummarizeStepPort>,
        step_exports_reader: Box<dyn ReadStepExportsPort>,
    ) -> Self {
        Self {
            job_environment_builder,
            container_preparer,
            step_path_prefixer,
            step_context_builder,
            step_executor,
            step_summarizer,
            step_exports_reader,
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

            let outcome = self.step_executor.execute(ExecuteStepRequest {
                step,
                context: &step_context,
                container: prepared.container.clone(),
                repo_path: request.repo_path,
                env: &step_env,
            });

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
