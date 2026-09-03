use std::collections::HashMap;

use super::{ContainerConfig, ContainerError, ExecResult, FileEntry, HostInfo, RunnerContext};

/// Outbound port for managing a container runtime (Docker, Podman, etc.).
///
/// Adapters implement this trait to provide platform-specific container
/// lifecycle operations: pulling images, creating containers, and querying
/// host information.
pub trait ContainerRuntimePort {
    /// Pull a container image from a registry.
    fn pull_image(&self, image: &str, platform: Option<&str>) -> Result<(), ContainerError>;

    /// Create and start a container from the given configuration.
    /// Returns a handle to the running container.
    fn create_container(
        &self,
        config: &ContainerConfig,
    ) -> Result<Box<dyn ContainerPort>, ContainerError>;

    /// Force-remove a container by name.
    ///
    /// Used to clean up stale containers from prior runs before creating a
    /// new container with the same name. Implementations SHOULD ignore
    /// errors when the container does not exist.
    fn remove_container(&self, name: &str) -> Result<(), ContainerError>;

    /// Stop a running container by name without removing it.
    ///
    /// The container and its filesystem are preserved; only the process is
    /// stopped. Images are never touched.
    fn stop_container(&self, name: &str) -> Result<(), ContainerError>;

    /// Return information about the host container runtime.
    fn get_host_info(&self) -> Result<HostInfo, ContainerError>;
}

/// Outbound port for interacting with a running container.
///
/// Returned by [`ContainerRuntimePort::create_container`]. Provides exec, file
/// transfer, inspection, and cleanup operations.
pub trait ContainerPort {
    /// Execute a command inside the container and return the result.
    ///
    /// `env` provides additional environment variables for this execution.
    /// These are merged on top of the container's base environment.
    fn exec(
        &self,
        cmd: &[String],
        workdir: Option<&str>,
        env: &HashMap<String, String>,
    ) -> Result<ExecResult, ContainerError>;

    /// Copy a file or directory into the container.
    fn copy_to(&self, container_path: &str, entries: &[FileEntry]) -> Result<(), ContainerError>;

    /// Copy a file or directory from the container.
    fn copy_from(&self, container_path: &str) -> Result<Vec<FileEntry>, ContainerError>;

    /// Remove the container (force-kill if running).
    fn remove(&self) -> Result<(), ContainerError>;

    /// Return runner context information for the container.
    fn get_runner_context(&self) -> Result<RunnerContext, ContainerError>;
}
