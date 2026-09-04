use crate::infrastructure::containers::{
    create_job_container_port::CreateJobContainerPort, workspace::CONTAINER_WORKSPACE,
};
use std::{collections::HashMap, error::Error, sync::Arc};

use crate::application::dtos::CreateJobContainerRequest;
use crate::application::dtos::RunnerContext;
use crate::application::ports::outbound::container_port::ContainerPort;
use crate::infrastructure::containers::ContainerConfig;
use crate::infrastructure::containers::ContainerRuntimePort;

/// Service that creates the container a job's steps run in, removing any
/// container left behind by an earlier run of the same job first.
pub struct CreateJobContainerService {
    runtime: Arc<dyn ContainerRuntimePort>,
}

impl CreateJobContainerService {
    pub fn new(runtime: Arc<dyn ContainerRuntimePort>) -> Self {
        Self { runtime }
    }
}

impl CreateJobContainerPort for CreateJobContainerService {
    fn execute(
        &self,
        request: CreateJobContainerRequest<'_>,
    ) -> Result<Arc<dyn ContainerPort>, Box<dyn Error>> {
        let _ = self.runtime.remove_container(request.legacy_container_name);
        let _ = self.runtime.remove_container(request.container_name);

        let container_config = ContainerConfig {
            image: request.image.to_string(),
            platform: None,
            env: HashMap::new(),
            binds: vec![format!(
                "{}:{}:Z",
                request.repo_path.display(),
                CONTAINER_WORKSPACE
            )],
            workdir: Some(CONTAINER_WORKSPACE.into()),
            cmd: Some(vec!["sleep".into(), "infinity".into()]),
            entrypoint: None,
            network: None,
            name: Some(request.container_name.to_string()),
            runner_context: RunnerContext::default(),
        };

        Ok(Arc::from(
            self.runtime
                .create_container(&container_config)
                .map_err(|e| format!("{:?}", e))?,
        ))
    }
}
