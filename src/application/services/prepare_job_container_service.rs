use std::{error::Error, process};

use crate::application::{
    dtos::{
        CreateJobContainerRequest, PrepareJobContainerRequest, PreparedJobContainer,
        PullJobImageRequest,
    },
    ports::inbound::{
        create_job_container_port::CreateJobContainerPort,
        prepare_job_container_port::PrepareJobContainerPort, pull_job_image_port::PullJobImagePort,
    },
};

/// Service that prepares a job's container: pulls the image the job needs and
/// creates the container its steps run in.
pub struct PrepareJobContainerService {
    image_puller: Box<dyn PullJobImagePort>,
    container_creator: Box<dyn CreateJobContainerPort>,
}

impl PrepareJobContainerService {
    pub fn new(
        image_puller: Box<dyn PullJobImagePort>,
        container_creator: Box<dyn CreateJobContainerPort>,
    ) -> Self {
        Self {
            image_puller,
            container_creator,
        }
    }
}

impl PrepareJobContainerPort for PrepareJobContainerService {
    fn execute(
        &self,
        request: PrepareJobContainerRequest<'_>,
    ) -> Result<PreparedJobContainer, Box<dyn Error>> {
        let image = self.image_puller.execute(PullJobImageRequest {
            runs_on: request.runs_on,
        })?;

        let container_name = format!("ephemeral-act-{}-{}", request.job_id, process::id());
        let legacy_container_name = format!("ephemeral-act-{}", request.job_id);

        let container = self.container_creator.execute(CreateJobContainerRequest {
            image: &image,
            container_name: &container_name,
            legacy_container_name: &legacy_container_name,
            repo_path: request.repo_path,
        })?;

        Ok(PreparedJobContainer {
            container,
            container_name,
        })
    }
}
