use std::fmt;

use crate::core::value_objects::ContainerEngine;

// ---------------------------------------------------------------------------
// Value objects
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActWorkflow(String);

impl ActWorkflow {
    pub fn new(path: String) -> Self {
        Self(path)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActJob(String);

impl ActJob {
    pub fn new(job: String) -> Self {
        Self(job)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActEvent(String);

impl ActEvent {
    pub fn new(event: String) -> Self {
        Self(event)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActInput {
    key: String,
    value: String,
}

impl ActInput {
    pub fn new(key: String, value: String) -> Self {
        Self { key, value }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

pub struct Secret(String);

impl Secret {
    pub fn new(value: String) -> Self {
        Self(value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(***)")
    }
}

impl Clone for Secret {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl PartialEq for Secret {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Secret {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActExtraArg(String);

impl ActExtraArg {
    pub fn new(arg: String) -> Self {
        Self(arg)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

// ---------------------------------------------------------------------------
// Aggregate
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActRunConfig {
    container_engine: ContainerEngine,
    workflow: Option<ActWorkflow>,
    job: Option<ActJob>,
    event: Option<ActEvent>,
    inputs: Vec<ActInput>,
    secrets: Vec<Secret>,
    extra_args: Vec<ActExtraArg>,
}

impl ActRunConfig {
    pub fn new(container_engine: ContainerEngine) -> Self {
        Self {
            container_engine,
            workflow: None,
            job: None,
            event: None,
            inputs: Vec::new(),
            secrets: Vec::new(),
            extra_args: Vec::new(),
        }
    }

    pub fn with_workflow(mut self, workflow: ActWorkflow) -> Self {
        self.workflow = Some(workflow);
        self
    }

    pub fn with_job(mut self, job: ActJob) -> Self {
        self.job = Some(job);
        self
    }

    pub fn with_event(mut self, event: ActEvent) -> Self {
        self.event = Some(event);
        self
    }

    pub fn add_input(mut self, input: ActInput) -> Self {
        self.inputs.push(input);
        self
    }

    pub fn add_secret(mut self, secret: Secret) -> Self {
        self.secrets.push(secret);
        self
    }

    pub fn add_extra_arg(mut self, arg: ActExtraArg) -> Self {
        self.extra_args.push(arg);
        self
    }

    pub fn container_engine(&self) -> &ContainerEngine {
        &self.container_engine
    }

    pub fn workflow(&self) -> Option<&ActWorkflow> {
        self.workflow.as_ref()
    }

    pub fn job(&self) -> Option<&ActJob> {
        self.job.as_ref()
    }

    pub fn event(&self) -> Option<&ActEvent> {
        self.event.as_ref()
    }

    pub fn inputs(&self) -> &[ActInput] {
        &self.inputs
    }

    pub fn secrets(&self) -> &[Secret] {
        &self.secrets
    }

    pub fn extra_args(&self) -> &[ActExtraArg] {
        &self.extra_args
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_config_starts_empty() {
        let config = ActRunConfig::new(ContainerEngine::Podman);
        assert_eq!(config.container_engine(), &ContainerEngine::Podman);
        assert!(config.workflow().is_none());
        assert!(config.job().is_none());
        assert!(config.event().is_none());
        assert!(config.inputs().is_empty());
        assert!(config.secrets().is_empty());
        assert!(config.extra_args().is_empty());
    }

    #[test]
    fn builder_adds_workflow_job_and_event() {
        let config = ActRunConfig::new(ContainerEngine::Docker)
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
        let config = ActRunConfig::new(ContainerEngine::Podman)
            .add_input(ActInput::new("environment".into(), "staging".into()))
            .add_extra_arg(ActExtraArg::new("--verbose".into()));

        assert_eq!(config.inputs()[0].key(), "environment");
        assert_eq!(config.inputs()[0].value(), "staging");
        assert_eq!(config.extra_args()[0].as_str(), "--verbose");
    }

    #[test]
    fn secret_debug_redacts_value() {
        let secret = Secret::new("my-token".into());
        let debug = format!("{:?}", secret);
        assert!(!debug.contains("my-token"));
        assert!(debug.contains("***"));
    }
}