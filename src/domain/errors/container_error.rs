/// Errors that can occur during container operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerError {
    /// No container runtime is available on this host.
    NotAvailable,

    /// Failed to pull the container image.
    ImagePullFailed(String, String),

    /// Failed to create the container.
    CreationFailed(String, String),

    /// Failed to execute a command inside the container.
    ExecutionFailed(String, String),

    /// Failed to copy files to/from the container.
    CopyFailed(String, String),

    /// Failed to remove the container.
    RemovalFailed(String, String),

    /// The requested platform is not supported by this runtime.
    UnsupportedPlatform(String),

    /// The container was not found.
    NotFound(String),

    /// An internal error occurred.
    Internal(String),
}
