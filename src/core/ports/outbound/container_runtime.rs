use std::collections::HashMap;

use thiserror::Error;

/// Configuration for creating a container.
#[derive(Debug, Clone)]
pub struct ContainerConfig {
    /// Docker image name (e.g. "catthehacker/ubuntu:act-latest")
    pub image: String,
    /// Target platform (e.g. "linux/amd64")
    pub platform: Option<String>,
    /// Environment variables injected into the container
    pub env: HashMap<String, String>,
    /// Volume binds in "host_path:container_path" format
    pub binds: Vec<String>,
    /// Working directory inside the container
    pub workdir: Option<String>,
    /// Command to run (overrides image CMD)
    pub cmd: Option<Vec<String>>,
    /// Entrypoint override
    pub entrypoint: Option<Vec<String>>,
    /// Network mode (e.g. "host", "bridge")
    pub network: Option<String>,
    /// Container name
    pub name: Option<String>,
}

/// Information about the host container runtime.
#[derive(Debug, Clone)]
pub struct HostInfo {
    /// Operating system (e.g. "linux")
    pub os: String,
    /// Architecture (e.g. "amd64")
    pub arch: String,
    /// Container engine version string
    pub engine_version: String,
}

/// Result of a container exec command.
#[derive(Debug, Clone)]
pub struct ExecResult {
    /// Process exit code
    pub exit_code: i64,
    /// Captured stdout
    pub stdout: String,
    /// Captured stderr
    pub stderr: String,
}

/// A file entry for copy operations.
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Path inside the container
    pub path: String,
    /// File content as raw bytes
    pub content: Vec<u8>,
    /// Unix file mode (e.g. 0o644)
    pub mode: u32,
}

/// Context describing the runner environment inside the container.
#[derive(Debug, Clone)]
pub struct RunnerContext {
    /// Workspace directory path
    pub workspace: String,
    /// Home directory path
    pub home: String,
    /// GitHub Actions action path
    pub action_path: String,
    /// Temp directory path
    pub temp: String,
    /// Tool cache directory path
    pub tool_cache: String,
    /// Environment variables visible inside the container
    pub env: HashMap<String, String>,
}

/// Errors that can occur during container operations.
#[derive(Debug, Error)]
pub enum ContainerError {
    /// No container runtime is available on this host.
    #[error("no container runtime available")]
    NotAvailable,

    /// Failed to pull the container image.
    #[error("failed to pull image '{0}': {1}")]
    ImagePullFailed(String, String),

    /// Failed to create the container.
    #[error("failed to create container '{0}': {1}")]
    CreationFailed(String, String),

    /// Failed to execute a command inside the container.
    #[error("execution failed in container '{0}': {1}")]
    ExecutionFailed(String, String),

    /// Failed to copy files to/from the container.
    #[error("copy failed in container '{0}': {1}")]
    CopyFailed(String, String),

    /// Failed to remove the container.
    #[error("failed to remove container '{0}': {1}")]
    RemovalFailed(String, String),

    /// The requested platform is not supported by this runtime.
    #[error("unsupported platform: {0}")]
    UnsupportedPlatform(String),

    /// The container was not found.
    #[error("container not found: {0}")]
    NotFound(String),

    /// An internal error occurred.
    #[error("internal error: {0}")]
    Internal(String),
}

/// Outbound port for managing a container runtime (Docker, Podman, etc.).
///
/// Adapters implement this trait to provide platform-specific container
/// lifecycle operations: pulling images, creating containers, and querying
/// host information.
pub trait ContainerRuntime {
    /// Pull a container image from a registry.
    fn pull_image(&self, image: &str, platform: Option<&str>) -> Result<(), ContainerError>;

    /// Create and start a container from the given configuration.
    /// Returns a handle to the running container.
    fn create_container(
        &self,
        config: &ContainerConfig,
    ) -> Result<Box<dyn Container>, ContainerError>;

    /// Return information about the host container runtime.
    fn get_host_info(&self) -> Result<HostInfo, ContainerError>;
}

/// Outbound port for interacting with a running container.
///
/// Returned by [`ContainerRuntime::create_container`]. Provides exec, file
/// transfer, inspection, and cleanup operations.
pub trait Container {
    /// Execute a command inside the container and return the result.
    fn exec(&self, cmd: &[String], workdir: Option<&str>) -> Result<ExecResult, ContainerError>;

    /// Copy a file or directory into the container.
    fn copy_to(&self, container_path: &str, entries: &[FileEntry]) -> Result<(), ContainerError>;

    /// Copy a file or directory from the container.
    fn copy_from(&self, container_path: &str) -> Result<Vec<FileEntry>, ContainerError>;

    /// Remove the container (force-kill if running).
    fn remove(&self) -> Result<(), ContainerError>;

    /// Return runner context information for the container.
    fn get_runner_context(&self) -> Result<RunnerContext, ContainerError>;
}