use serde::Deserialize;

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
