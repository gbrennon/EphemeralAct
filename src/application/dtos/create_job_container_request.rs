use std::path::Path;

/// Request DTO for the
/// [`CreateJobContainerPort`](crate::application::ports::inbound::create_job_container_port::CreateJobContainerPort)
/// inbound port.
pub struct CreateJobContainerRequest<'a> {
    /// Image the container is created from.
    pub image: &'a str,
    /// Name the new container is given.
    pub container_name: &'a str,
    /// Name older releases gave the same job's container.
    pub legacy_container_name: &'a str,
    /// Repository directory mounted into the container as the workspace.
    pub repo_path: &'a Path,
}
