use serde::Deserialize;

/// Credentials for pulling a container image from a private registry.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ContainerCredentials {
    pub username: String,
    pub password: String,
}
