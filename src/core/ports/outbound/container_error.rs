use thiserror::Error;

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
