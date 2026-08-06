use serde::Deserialize;
use std::collections::HashMap;

pub use crate::core::workflow::event::On;
pub use crate::core::workflow::job::Job;

/// Represents a parsed GitHub Actions workflow file.
///
/// Maps to the top-level structure of a workflow YAML file.
/// Supports all standard fields including `name`, `on`, `env`, `jobs`,
/// `defaults`, and `permissions`.
///
/// # Examples
///
/// ```
/// use ephemeral_act::core::workflow::Workflow;
///
/// let yaml = r#"
/// name: CI
/// on: push
/// jobs:
///   build:
///     runs-on: ubuntu-latest
///     steps:
///       - run: echo hello
/// "#;
/// let wf: Workflow = serde_yaml::from_str(yaml).unwrap();
/// assert_eq!(wf.name.as_deref(), Some("CI"));
/// assert_eq!(wf.jobs.len(), 1);
/// ```
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Workflow {
    /// The name of the workflow displayed on GitHub's actions page.
    pub name: Option<String>,

    /// The name of the workflow file (set after parsing, not from YAML).
    #[serde(skip)]
    pub file: Option<String>,

    /// The event(s) that trigger this workflow.
    #[serde(default)]
    pub on: On,

    /// Environment variables available to all jobs and steps.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// The jobs that make up this workflow.
    #[serde(default)]
    pub jobs: HashMap<String, Job>,

    /// Default settings applied to all jobs in the workflow.
    #[serde(default)]
    pub defaults: Option<Defaults>,

    /// Permissions for the `GITHUB_TOKEN`.
    #[serde(default)]
    pub permissions: Option<Permissions>,

    /// Concurrency group to limit parallel runs.
    #[serde(default)]
    pub concurrency: Option<Concurrency>,
}

/// Default settings for all jobs in a workflow.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Defaults {
    /// Default run settings (shell, working-directory).
    pub run: Option<RunDefaults>,
}

/// Default run settings for shell and working directory.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct RunDefaults {
    /// Default shell for run steps.
    pub shell: Option<String>,

    /// Default working directory for run steps.
    #[serde(rename = "working-directory")]
    pub working_directory: Option<String>,
}

/// Permissions for the `GITHUB_TOKEN` in a workflow or job.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Permissions {
    pub actions: Option<String>,
    pub checks: Option<String>,
    pub contents: Option<String>,
    pub deployments: Option<String>,
    pub issues: Option<String>,
    pub packages: Option<String>,
    pub pages: Option<String>,
    #[serde(rename = "pull-requests")]
    pub pull_requests: Option<String>,
    #[serde(rename = "repository-projects")]
    pub repository_projects: Option<String>,
    #[serde(rename = "security-events")]
    pub security_events: Option<String>,
    pub statuses: Option<String>,
}

/// Concurrency configuration to limit parallel workflow runs.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Concurrency {
    /// The concurrency group name.
    pub group: String,

    /// Whether to cancel in-progress runs in the same group.
    #[serde(rename = "cancel-in-progress")]
    pub cancel_in_progress: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_workflow() {
        let yaml = "on: push\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - run: echo hello\n";
        let wf: Workflow = serde_yaml::from_str(yaml).unwrap();
        assert!(wf.name.is_none());
        assert_eq!(wf.jobs.len(), 1);
        assert!(wf.jobs.contains_key("build"));
    }

    #[test]
    fn parse_workflow_with_name_and_env() {
        let yaml = r#"
name: CI
on: [push, pull_request]
env:
  RUST_BACKTRACE: "1"
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - run: cargo test
"#;
        let wf: Workflow = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(wf.name.as_deref(), Some("CI"));
        assert_eq!(wf.env.get("RUST_BACKTRACE").map(|s| s.as_str()), Some("1"));
    }

    #[test]
    fn parse_workflow_with_defaults() {
        let yaml = r#"
on: push
defaults:
  run:
    shell: bash
    working-directory: ./src
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - run: make
"#;
        let wf: Workflow = serde_yaml::from_str(yaml).unwrap();
        let defaults = wf.defaults.unwrap();
        let run_defaults = defaults.run.unwrap();
        assert_eq!(run_defaults.shell.as_deref(), Some("bash"));
        assert_eq!(run_defaults.working_directory.as_deref(), Some("./src"));
    }
}

pub mod event;
pub mod job;
pub mod step;
pub mod strategy;

pub use step::Step;
pub use strategy::Strategy;
