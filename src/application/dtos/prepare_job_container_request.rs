use std::path::Path;

/// Request DTO for the
/// [`PrepareJobContainerPort`](crate::application::ports::inbound::prepare_job_container_port::PrepareJobContainerPort)
/// inbound port.
pub struct PrepareJobContainerRequest<'a> {
    /// Identifier of the job the container is prepared for.
    pub job_id: &'a str,
    /// Runner label the job declared, when it declared one.
    pub runs_on: Option<&'a str>,
    /// Repository directory mounted into the container as the workspace.
    pub repo_path: &'a Path,
}
