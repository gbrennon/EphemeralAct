use std::collections::HashMap;

use serde::Deserialize;

use super::{Concurrency, Defaults, Job, On, Permissions};

/// Represents a parsed GitHub Actions workflow file.
///
/// Maps to the top-level structure of a workflow YAML file.
/// Supports all standard fields including `name`, `on`, `env`, `jobs`,
/// `defaults`, and `permissions`.
///
/// # Examples
///
/// ```
/// use ephact::core::workflow::Workflow;
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
