use std::collections::HashMap;

use serde::Deserialize;

use super::ContainerCredentials;

/// Configuration for a container used by a job or service.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ContainerConfig {
    /// The Docker image to use.
    pub image: String,

    /// Credentials for pulling the image from a private registry.
    #[serde(default)]
    pub credentials: Option<ContainerCredentials>,

    /// Environment variables for the container.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Ports to expose on the container.
    #[serde(default)]
    pub ports: Vec<String>,

    /// Volumes to mount in the container.
    #[serde(default)]
    pub volumes: Vec<String>,

    /// Additional options passed to `docker create`.
    #[serde(default)]
    pub options: Option<String>,
}
