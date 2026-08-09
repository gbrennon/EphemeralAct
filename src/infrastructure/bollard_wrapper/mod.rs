/// Centralized bollard API wrapper.
///
/// Bollard's `Docker` client struct works with any Docker-compatible API
/// (Docker, Podman, etc.). This module re-exports it under the generic name
/// [`Client`] and centralizes all bollard type imports so adapter code
/// doesn't couple directly to bollard's module structure.
///
/// # Naming
///
/// | Bollard name | Wrapper export | Reason |
/// |---|---|---|
/// | `bollard::Docker` | [`Client`] | Works with Docker and Podman |
/// | `bollard::auth::DockerCredentials` | [`AuthCredentials`] | Not Docker-specific |
pub use bollard::{self, Docker as Client};

/// Re-exported bollard types used across runtime and container adapters.
pub mod types {
    pub use bollard::{
        container::LogOutput,
        exec::{CreateExecOptions, StartExecOptions, StartExecResults},
        models::{ContainerCreateBody, HostConfig},
        query_parameters::{
            CreateContainerOptionsBuilder, CreateImageOptionsBuilder,
            DownloadFromContainerOptionsBuilder, InspectContainerOptions, RemoveContainerOptions,
            StartContainerOptions, UploadToContainerOptionsBuilder,
        },
    };
}

/// bollard's default API version constant.
pub use bollard::API_DEFAULT_VERSION;
/// bollard's `DockerCredentials` under a runtime-agnostic name.
pub use bollard::auth::DockerCredentials as AuthCredentials;
/// Re-exported bollard free functions.
pub use bollard::body_full;
