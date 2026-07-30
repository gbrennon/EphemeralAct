use crate::core::errors::CoreError;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerEngine {
    Podman,
    Docker,
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
    /// # use ephemeral_act::core::value_objects::ContainerEngine;
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
}
