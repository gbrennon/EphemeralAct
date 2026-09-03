use crate::application::{
    constants::{CONTAINER_WORKSPACE, GITHUB_ENV_FILE, GITHUB_PATH_FILE},
    dtos::{BuildJobEnvironmentRequest, BuildJobEnvironmentResponse},
    ports::outbound::build_job_environment_port::BuildJobEnvironmentPort,
};

/// `PATH` a job runs with when neither the workflow nor the job declares one.
const DEFAULT_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

/// Infrastructure adapter that builds the execution environment for a job's container
/// following the GitHub Actions specification: merging workflow and job environments,
/// and setting `GITHUB_PATH`, `GITHUB_ENV`, `GITHUB_WORKSPACE`, and default `PATH`.
pub struct GitHubJobEnvironmentAdapter;

impl GitHubJobEnvironmentAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GitHubJobEnvironmentAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildJobEnvironmentPort for GitHubJobEnvironmentAdapter {
    fn execute(&self, request: BuildJobEnvironmentRequest<'_>) -> BuildJobEnvironmentResponse {
        let mut env = request.workflow.env.clone();
        for (key, value) in request.job_env {
            env.insert(key.clone(), value.clone());
        }

        env.insert("GITHUB_PATH".into(), GITHUB_PATH_FILE.into());
        env.insert("GITHUB_ENV".into(), GITHUB_ENV_FILE.into());
        env.insert("GITHUB_WORKSPACE".into(), CONTAINER_WORKSPACE.into());
        env.entry("PATH".to_string())
            .or_insert_with(|| DEFAULT_PATH.to_string());
        BuildJobEnvironmentResponse { env }
    }
}
