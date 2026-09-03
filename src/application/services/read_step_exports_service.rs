use crate::application::{
    dtos::{
        ReadStepEnvExportsRequest, ReadStepExportsRequest, ReadStepPathExportsRequest, StepExports,
    },
    ports::inbound::{
        read_step_env_exports_port::ReadStepEnvExportsPort,
        read_step_exports_port::ReadStepExportsPort,
        read_step_path_exports_port::ReadStepPathExportsPort,
    },
};

/// Service that reads everything a step exported to the steps that follow it.
pub struct ReadStepExportsService {
    path_reader: Box<dyn ReadStepPathExportsPort>,
    env_reader: Box<dyn ReadStepEnvExportsPort>,
}

impl ReadStepExportsService {
    pub fn new(
        path_reader: Box<dyn ReadStepPathExportsPort>,
        env_reader: Box<dyn ReadStepEnvExportsPort>,
    ) -> Self {
        Self {
            path_reader,
            env_reader,
        }
    }
}

impl ReadStepExportsPort for ReadStepExportsService {
    fn execute(&self, request: ReadStepExportsRequest<'_>) -> StepExports {
        StepExports {
            path_additions: self.path_reader.execute(ReadStepPathExportsRequest {
                container: request.container,
            }),
            env: self.env_reader.execute(ReadStepEnvExportsRequest {
                container: request.container,
            }),
        }
    }
}
