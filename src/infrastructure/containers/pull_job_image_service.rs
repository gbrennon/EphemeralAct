use crate::infrastructure::containers::pull_job_image_port::PullJobImagePort;
use std::{error::Error, sync::Arc};

use crate::{
    application::dtos::PullJobImageRequest,
    infrastructure::{containers::ContainerRuntimePort, images::ImageMapperPort},
};

/// Runner label assumed when a job declares none.
const DEFAULT_RUNNER_LABEL: &str = "ubuntu-latest";

/// Service that pulls the image a job runs in, falling back to the mapper's
/// default image when the mapped one cannot be pulled.
pub struct PullJobImageService {
    runtime: Arc<dyn ContainerRuntimePort>,
    image_mapper: Arc<dyn ImageMapperPort>,
}

impl PullJobImageService {
    pub fn new(
        runtime: Arc<dyn ContainerRuntimePort>,
        image_mapper: Arc<dyn ImageMapperPort>,
    ) -> Self {
        Self {
            runtime,
            image_mapper,
        }
    }
}

impl PullJobImagePort for PullJobImageService {
    fn execute(&self, request: PullJobImageRequest<'_>) -> Result<String, Box<dyn Error>> {
        let runs_on = request.runs_on.unwrap_or(DEFAULT_RUNNER_LABEL);
        let mut image = self.image_mapper.map(runs_on);

        if self.runtime.pull_image(&image, None).is_err() {
            image = self.image_mapper.fallback();
            self.runtime
                .pull_image(&image, None)
                .map_err(|e| format!("{:?}", e))?;
        }

        Ok(image)
    }
}
