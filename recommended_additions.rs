// File: src/core/value_objects/container_daemon_socket.rs
// NEW FILE — Add this value object to capture container socket configuration

use crate::core::value_objects::ContainerEngine;
use std::env;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerDaemonSocket(String);

impl ContainerDaemonSocket {
    /// Creates a socket from an explicit string.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::ContainerDaemonSocket;
    /// let socket = ContainerDaemonSocket::new("unix:///run/user/1000/podman/podman.sock".into());
    /// assert_eq!(socket.as_str(), "unix:///run/user/1000/podman/podman.sock");
    /// ```
    pub fn new(socket: String) -> Self {
        Self(socket)
    }

    /// Resolves the container daemon socket for the given engine.
    ///
    /// Checks environment variables (`PODMAN_SOCK` or `DOCKER_SOCK`) first,
    /// then falls back to platform defaults.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::{ContainerEngine, ContainerDaemonSocket};
    /// let socket = ContainerDaemonSocket::for_engine(&ContainerEngine::Podman);
    /// assert!(socket.as_str().contains("podman"));
    /// ```
    pub fn for_engine(engine: &ContainerEngine) -> Self {
        let socket = match engine {
            ContainerEngine::Podman => {
                env::var("PODMAN_SOCK").unwrap_or_else(|_| {
                    let uid = unsafe { libc::getuid() };
                    format!("unix:///run/user/{}/podman/podman.sock", uid)
                })
            }
            ContainerEngine::Docker => {
                env::var("DOCKER_SOCK")
                    .unwrap_or_else(|_| "unix:///var/run/docker.sock".to_string())
            }
        };

        Self(socket)
    }

    /// Returns the socket path as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_socket_string() {
        let socket = ContainerDaemonSocket::new("unix:///tmp/test.sock".into());
        assert_eq!(socket.as_str(), "unix:///tmp/test.sock");
    }

    #[test]
    fn for_engine_podman_contains_podman() {
        let socket = ContainerDaemonSocket::for_engine(&ContainerEngine::Podman);
        assert!(socket.as_str().contains("podman"));
    }

    #[test]
    fn for_engine_docker_contains_var_run() {
        let socket = ContainerDaemonSocket::for_engine(&ContainerEngine::Docker);
        assert!(socket.as_str().contains("docker"));
    }
}

// ============================================================================
// File: src/core/value_objects/mod.rs
// UPDATE — Add the new value object to exports

pub mod act_event;
pub mod act_extra_arg;
pub mod act_input;
pub mod act_job;
pub mod act_workflow;
pub mod cleanup_policy;
pub mod container_engine;
pub mod container_daemon_socket;  // ADD THIS LINE
pub mod repo_path;
pub mod repository_name;
pub mod secret;

pub use self::act_event::ActEvent;
pub use self::act_extra_arg::ActExtraArg;
pub use self::act_input::ActInput;
pub use self::act_job::ActJob;
pub use self::act_workflow::ActWorkflow;
pub use self::cleanup_policy::CleanupPolicy;
pub use self::container_engine::ContainerEngine;
pub use self::container_daemon_socket::ContainerDaemonSocket;  // ADD THIS LINE
pub use self::repo_path::{GitDirKind, RepoPath};
pub use self::repository_name::RepositoryName;
pub use self::secret::Secret;

// ============================================================================
// File: src/core/act_run_config.rs
// UPDATE — Add socket field to ActRunConfig

use crate::core::value_objects::{
    ActEvent, ActExtraArg, ActInput, ActJob, ActWorkflow, ContainerEngine,
    ContainerDaemonSocket,  // ADD THIS IMPORT
    Secret,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActRunConfig {
    container_engine: ContainerEngine,
    container_daemon_socket: ContainerDaemonSocket,  // ADD THIS FIELD
    workflow: Option<ActWorkflow>,
    job: Option<ActJob>,
    event: Option<ActEvent>,
    inputs: Vec<ActInput>,
    secrets: Vec<Secret>,
    extra_args: Vec<ActExtraArg>,
    rm: bool,
    bind: bool,
}

impl ActRunConfig {
    /// Creates a new config with the given container engine and sensible defaults.
    ///
    /// The container daemon socket is auto-detected from the engine and environment
    /// variables. Both `--rm` and `--bind` default to `true`, matching the act CLI's
    /// typical invocation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::ContainerEngine;
    /// # use ephemeral_act::core::ActRunConfig;
    /// let config = ActRunConfig::new(ContainerEngine::Podman);
    /// assert!(config.rm());
    /// assert!(config.bind());
    /// // Socket is auto-detected from PODMAN_SOCK env var or default
    /// ```
    pub fn new(container_engine: ContainerEngine) -> Self {
        let container_daemon_socket = ContainerDaemonSocket::for_engine(&container_engine);

        Self {
            container_engine,
            container_daemon_socket,
            workflow: None,
            job: None,
            event: None,
            inputs: Vec::new(),
            secrets: Vec::new(),
            extra_args: Vec::new(),
            rm: true,
            bind: true,
        }
    }

    // ... all the with_* builder methods remain unchanged ...

    /// Returns the container engine.
    pub fn container_engine(&self) -> &ContainerEngine {
        &self.container_engine
    }

    /// Returns the container daemon socket path.
    pub fn container_daemon_socket(&self) -> &ContainerDaemonSocket {
        &self.container_daemon_socket
    }

    /// Returns the workflow, if set.
    pub fn workflow(&self) -> Option<&ActWorkflow> {
        self.workflow.as_ref()
    }

    // ... rest of getters unchanged ...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_config_auto_resolves_socket() {
        let config = ActRunConfig::new(ContainerEngine::Podman);
        assert_eq!(config.container_engine(), &ContainerEngine::Podman);
        // Socket should contain "podman"
        assert!(config.container_daemon_socket().as_str().contains("podman"));
    }

    #[test]
    fn new_config_starts_with_defaults() {
        let config = ActRunConfig::new(ContainerEngine::Podman);
        assert!(config.workflow().is_none());
        assert!(config.job().is_none());
        assert!(config.event().is_none());
        assert!(config.inputs().is_empty());
        assert!(config.secrets().is_empty());
        assert!(config.extra_args().is_empty());
        assert!(config.rm());
        assert!(config.bind());
    }

    // ... other tests unchanged ...
}

// ============================================================================
// File: src/core/mod.rs
// UPDATE — Export the new value object

pub mod act_run_config;
pub mod ephemeral_repository;
pub mod errors;
pub mod repository;
pub mod value_objects;

pub use self::act_run_config::ActRunConfig;
pub use self::ephemeral_repository::{EphemeralRepository, TempDirTemplate};
pub use self::errors::CoreError;
pub use self::repository::Repository;
pub use self::value_objects::{
    ActEvent, ActExtraArg, ActInput, ActJob, ActWorkflow, CleanupPolicy, ContainerEngine,
    ContainerDaemonSocket,  // ADD THIS LINE
    GitDirKind, RepoPath, RepositoryName, Secret,
};

// ============================================================================
// Service-layer usage example (pseudo-code)
// This shows how the services layer would use the enhanced core model to
// build the complete act CLI command.

// use ephemeral_act::core::{ActRunConfig, Repository, EphemeralRepository};
// use std::process::Command;

// let source = Repository::new(repo_path, repo_name);
// let ephemeral = EphemeralRepository::new(&source, CleanupPolicy::CleanupOnExit);
// let config = ActRunConfig::new(ContainerEngine::Podman)
//     .with_workflow(ActWorkflow::new(".github/workflows/ci.yml".into()))
//     .with_job(ActJob::new("test".into()));

// // Build the complete act command from core model
// let mut cmd = Command::new("act");
// cmd.arg("--container-daemon-socket")
//    .arg(config.container_daemon_socket().as_str());

// if let Some(wf) = config.workflow() {
//     cmd.arg("-w").arg(wf.as_str());
// }
// if let Some(j) = config.job() {
//     cmd.arg("-j").arg(j.as_str());
// }
// if let Some(e) = config.event() {
//     cmd.arg(e.as_str());  // Positional
// }

// for input in config.inputs() {
//     cmd.arg("-i")
//        .arg(format!("{}={}", input.key(), input.value()));
// }

// if config.rm() {
//     cmd.arg("--rm");
// }
// if config.bind() {
//     cmd.arg("--bind");
// }

// // ... worktree conversion, cleanup, execution handled by services ...
