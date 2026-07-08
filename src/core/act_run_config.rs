use crate::core::value_objects::{
    ActEvent, ActExtraArg, ActInput, ActJob, ActWorkflow, ContainerDaemonSocket, ContainerEngine,
    Secret,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActRunConfig {
    container_daemon_socket: ContainerDaemonSocket,
    container_engine: ContainerEngine,
    workflow: Option<ActWorkflow>,
    job: Option<ActJob>,
    event: Option<ActEvent>,
    inputs: Vec<ActInput>,
    secrets: Vec<Secret>,
    extra_args: Vec<ActExtraArg>,
    rm: bool,
    bind: bool,
}

/// Constructors for [`ActRunConfig`].
impl ActRunConfig {
    /// Creates a new config with the given container engine, daemon socket, and
    /// sensible defaults.
    ///
    /// Both `--rm` and `--bind` default to `true`, matching the act CLI's typical
    /// invocation.
    ///
    /// # Examples
    ///
    /// ```
    /// # use ephemeral_act::core::value_objects::{ContainerDaemonSocket, ContainerEngine};
    /// # use ephemeral_act::core::ActRunConfig;
    /// let config = ActRunConfig::new(
    ///     ContainerEngine::Podman,
    ///     ContainerDaemonSocket::new("unix:///run/podman/podman.sock".into()),
    /// );
    /// assert!(config.rm());
    pub fn new(
        container_engine: ContainerEngine,
        container_daemon_socket: ContainerDaemonSocket,
    ) -> Self {
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

    /// Sets the workflow file to run (`-w`).
    pub fn with_workflow(mut self, workflow: ActWorkflow) -> Self {
        self.workflow = Some(workflow);
        self
    }

    /// Sets the specific job to run within the workflow (`-j`).
    pub fn with_job(mut self, job: ActJob) -> Self {
        self.job = Some(job);
        self
    }

    /// Sets the GitHub event to simulate (`-e`).
    pub fn with_event(mut self, event: ActEvent) -> Self {
        self.event = Some(event);
        self
    }

    /// Adds an input variable (`-i KEY=VALUE`).
    pub fn add_input(mut self, input: ActInput) -> Self {
        self.inputs.push(input);
        self
    }

    /// Adds a secret (`-s KEY=VALUE`).
    pub fn add_secret(mut self, secret: Secret) -> Self {
        self.secrets.push(secret);
        self
    }

    /// Adds an extra argument passed after `--` to act.
    pub fn add_extra_arg(mut self, arg: ActExtraArg) -> Self {
        self.extra_args.push(arg);
        self
    }

    /// Sets whether `--rm` is passed to act (default: `true`).
    pub fn with_rm(mut self, rm: bool) -> Self {
        self.rm = rm;
        self
    }

    /// Sets the container daemon socket path (`--container-daemon-socket`).
    pub fn with_container_daemon_socket(mut self, socket: ContainerDaemonSocket) -> Self {
        self.container_daemon_socket = socket;
        self
    }

    /// Returns the container daemon socket path.
    pub fn container_daemon_socket(&self) -> &ContainerDaemonSocket {
        &self.container_daemon_socket
    }
    /// Sets whether `--bind` is passed to act (default: `true`).
    pub fn with_bind(mut self, bind: bool) -> Self {
        self.bind = bind;
        self
    }

    /// Returns the container engine.
    pub fn container_engine(&self) -> &ContainerEngine {
        &self.container_engine
    }

    /// Returns the workflow, if set.
    pub fn workflow(&self) -> Option<&ActWorkflow> {
        self.workflow.as_ref()
    }

    /// Returns the job, if set.
    pub fn job(&self) -> Option<&ActJob> {
        self.job.as_ref()
    }

    /// Returns the event, if set.
    pub fn event(&self) -> Option<&ActEvent> {
        self.event.as_ref()
    }

    /// Returns all input variables.
    pub fn inputs(&self) -> &[ActInput] {
        &self.inputs
    }

    /// Returns all secrets.
    pub fn secrets(&self) -> &[Secret] {
        &self.secrets
    }

    /// Returns all extra arguments.
    pub fn extra_args(&self) -> &[ActExtraArg] {
        &self.extra_args
    }

    /// Returns whether `--rm` is enabled.
    pub fn rm(&self) -> bool {
        self.rm
    }

    /// Returns whether `--bind` is enabled.
    pub fn bind(&self) -> bool {
        self.bind
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_config_starts_with_defaults() {
        let config = ActRunConfig::new(
            ContainerEngine::Podman,
            ContainerDaemonSocket::new("unix:///tmp/test.sock".into()),
        );
        assert_eq!(config.container_engine(), &ContainerEngine::Podman);
        assert_eq!(
            config.container_daemon_socket().as_str(),
            "unix:///tmp/test.sock"
        );
        assert!(config.workflow().is_none());
        assert!(config.job().is_none());
        assert!(config.event().is_none());
        assert!(config.inputs().is_empty());
        assert!(config.secrets().is_empty());
        assert!(config.extra_args().is_empty());
        assert!(config.rm());
        assert!(config.bind());
    }

    #[test]
    fn builder_adds_workflow_job_and_event() {
        let config = ActRunConfig::new(
            ContainerEngine::Docker,
            ContainerDaemonSocket::new("unix:///tmp/test.sock".into()),
        )
        .with_workflow(ActWorkflow::new(".github/workflows/ci.yml".into()))
        .with_job(ActJob::new("test".into()))
        .with_event(ActEvent::new("push".into()));

        assert_eq!(
            config.workflow().unwrap().as_str(),
            ".github/workflows/ci.yml"
        );
        assert_eq!(config.job().unwrap().as_str(), "test");
        assert_eq!(config.event().unwrap().as_str(), "push");
    }

    #[test]
    fn builder_adds_inputs_and_extra_args() {
        let config = ActRunConfig::new(
            ContainerEngine::Podman,
            ContainerDaemonSocket::new("unix:///tmp/test.sock".into()),
        )
        .add_input(ActInput::new("environment".into(), "staging".into()))
        .add_extra_arg(ActExtraArg::new("--verbose".into()));

        assert_eq!(config.inputs()[0].key(), "environment");
        assert_eq!(config.inputs()[0].value(), "staging");
        assert_eq!(config.extra_args()[0].as_str(), "--verbose");
    }

    #[test]
    fn with_rm_overrides_default() {
        let config = ActRunConfig::new(
            ContainerEngine::Podman,
            ContainerDaemonSocket::new("unix:///tmp/test.sock".into()),
        )
        .with_rm(false);
        assert!(!config.rm());
    }

    #[test]
    fn with_bind_overrides_default() {
        let config = ActRunConfig::new(
            ContainerEngine::Podman,
            ContainerDaemonSocket::new("unix:///tmp/test.sock".into()),
        )
        .with_bind(false);
        assert!(!config.bind());
    }

    #[test]
    fn new_config_stores_socket() {
        let config = ActRunConfig::new(
            ContainerEngine::Podman,
            ContainerDaemonSocket::new("unix:///run/user/1000/podman/podman.sock".into()),
        );
        assert_eq!(
            config.container_daemon_socket().as_str(),
            "unix:///run/user/1000/podman/podman.sock"
        );
    }
}
