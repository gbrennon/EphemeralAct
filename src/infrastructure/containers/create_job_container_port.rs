use std::sync::Arc;

use crate::application::dtos::CreateJobContainerRequest;
use crate::application::ports::outbound::container_port::ContainerPort;

/// Inbound port for creating the container a job's steps run in.
pub trait CreateJobContainerPort: Send + Sync {
    /// Removes any stale container of the same job and creates a fresh one.
    fn execute(
        &self,
        request: CreateJobContainerRequest<'_>,
    ) -> Result<Arc<dyn ContainerPort>, Box<dyn std::error::Error>>;
}
