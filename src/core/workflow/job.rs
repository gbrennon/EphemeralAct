use serde::Deserialize;
use std::collections::HashMap;

use super::Step;

/// A job in a GitHub Actions workflow.
///
/// Jobs run in parallel by default but can be sequenced with `needs`.
/// Each job runs on a fresh virtual environment specified by `runs_on`.
///
/// # Examples
///
/// ```
/// use ephemeral_act::core::workflow::Job;
///
/// let yaml = r#"
/// runs-on: ubuntu-latest
/// steps:
///   - run: echo hello
/// "#;
/// let job: Job = serde_yaml::from_str(yaml).unwrap();
/// assert_eq!(job.runs_on.as_deref(), Some("ubuntu-latest"));
/// ```
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Job {
    /// The name of the job displayed on GitHub.
    pub name: Option<String>,

    /// The type of machine to run the job on (e.g. `ubuntu-latest`).
    #[serde(rename = "runs-on")]
    pub runs_on: Option<String>,

    /// The sequence of steps to execute.
    #[serde(default)]
    pub steps: Vec<Step>,

    /// Jobs that must complete successfully before this job runs.
    #[serde(default)]
    pub needs: Vec<String>,

    /// An expression that determines whether the job runs.
    #[serde(rename = "if")]
    pub r#if: Option<String>,

    /// A matrix strategy to generate multiple job runs.
    #[serde(default)]
    pub strategy: Option<super::Strategy>,

    /// Environment variables scoped to this job.
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Container to run the job inside.
    #[serde(default)]
    pub container: Option<ContainerConfig>,

    /// Service containers to run alongside the job.
    #[serde(default)]
    pub services: HashMap<String, ContainerConfig>,

    /// Outputs produced by this job (for dependent jobs).
    #[serde(default)]
    pub outputs: HashMap<String, String>,

    /// Input parameters passed via `workflow_call`.
    #[serde(default)]
    pub with: Option<serde_yaml::Value>,

    /// Secrets available to this job.
    #[serde(default)]
    pub secrets: Option<serde_yaml::Value>,

    /// Maximum number of minutes to let the job run.
    #[serde(rename = "timeout-minutes")]
    pub timeout_minutes: Option<f64>,

    /// Whether to continue the workflow even if this job fails.
    #[serde(rename = "continue-on-error")]
    pub continue_on_error: Option<String>,

    /// Permissions override for this job.
    #[serde(default)]
    pub permissions: Option<super::Permissions>,

    /// Concurrency override for this job.
    #[serde(default)]
    pub concurrency: Option<super::Concurrency>,
}

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

/// Credentials for pulling a container image from a private registry.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ContainerCredentials {
    pub username: String,
    pub password: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_job() {
        let yaml = "runs-on: ubuntu-latest\nsteps:\n  - run: echo hello\n";
        let job: Job = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(job.runs_on.as_deref(), Some("ubuntu-latest"));
        assert_eq!(job.steps.len(), 1);
    }

    #[test]
    fn parse_job_with_needs_and_if() {
        let yaml = r#"
runs-on: ubuntu-latest
needs: [build, lint]
if: github.ref == 'refs/heads/main'
steps:
  - run: echo deploy
"#;
        let job: Job = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(job.needs, vec!["build", "lint"]);
        assert_eq!(job.r#if.as_deref(), Some("github.ref == 'refs/heads/main'"));
    }

    #[test]
    fn parse_job_with_container() {
        let yaml = r#"
runs-on: ubuntu-latest
container:
  image: node:18
  env:
    NODE_ENV: test
steps:
  - run: npm test
"#;
        let job: Job = serde_yaml::from_str(yaml).unwrap();
        let container = job.container.unwrap();
        assert_eq!(container.image, "node:18");
        assert_eq!(
            container.env.get("NODE_ENV").map(|s| s.as_str()),
            Some("test")
        );
    }

    #[test]
    fn parse_job_with_timeout() {
        let yaml = r#"
runs-on: ubuntu-latest
timeout-minutes: 30
steps:
  - run: sleep 9999
"#;
        let job: Job = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(job.timeout_minutes, Some(30.0));
    }
}
