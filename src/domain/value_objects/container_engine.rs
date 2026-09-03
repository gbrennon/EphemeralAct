use std::str::FromStr;

use crate::domain::errors::core_error::CoreError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerEngine {
    Podman,
    Docker,
}

impl ContainerEngine {
    /// Returns the CLI name for the container engine.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephact::domain::value_objects::ContainerEngine;
    /// assert_eq!(ContainerEngine::Podman.as_str(), "podman");
    /// assert_eq!(ContainerEngine::Docker.as_str(), "docker");
    /// ```
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Podman => "podman",
            Self::Docker => "docker",
        }
    }
}

impl FromStr for ContainerEngine {
    type Err = CoreError;

    /// Parses a container engine from its CLI name.
    ///
    /// Returns [`CoreError::UnknownContainerEngine`] for unrecognized values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::str::FromStr;
    /// # use ephact::domain::value_objects::ContainerEngine;
    /// assert!(ContainerEngine::from_str("podman").is_ok());
    /// assert!(ContainerEngine::from_str("docker").is_ok());
    /// assert!(ContainerEngine::from_str("lxc").is_err());
    /// ```
    fn from_str(engine: &str) -> Result<Self, Self::Err> {
        match engine {
            "podman" => Ok(Self::Podman),
            "docker" => Ok(Self::Docker),
            other => Err(CoreError::UnknownContainerEngine(other.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_str_podman_returns_podman() {
        assert_eq!(
            ContainerEngine::from_str("podman"),
            Ok(ContainerEngine::Podman)
        );
    }

    #[test]
    fn from_str_docker_returns_docker() {
        assert_eq!(
            ContainerEngine::from_str("docker"),
            Ok(ContainerEngine::Docker)
        );
    }

    #[test]
    fn from_str_unknown_returns_error() {
        let result = ContainerEngine::from_str("lxc");
        assert!(matches!(result, Err(CoreError::UnknownContainerEngine(_))));
    }

    #[test]
    fn as_str_podman() {
        assert_eq!(ContainerEngine::Podman.as_str(), "podman");
    }

    #[test]
    fn as_str_docker() {
        assert_eq!(ContainerEngine::Docker.as_str(), "docker");
    }
}
