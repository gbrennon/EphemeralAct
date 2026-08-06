use serde::Serialize;
use std::collections::HashMap;

/// A GitHub Actions event that triggers a workflow.
///
/// Each variant corresponds to a webhook event that GitHub sends.
/// The event payload is serialized to JSON and made available
/// via the `github` context in expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Push(Box<PushPayload>),
    PullRequest(Box<PullRequestPayload>),
    WorkflowDispatch(Box<WorkflowDispatchPayload>),
    Schedule,
    Release(Box<ReleasePayload>),
    Issues(Box<IssuesPayload>),
    IssueComment(Box<IssueCommentPayload>),
    Create(Box<CreatePayload>),
    Delete(Box<DeletePayload>),
    Fork(Box<ForkPayload>),
    Gollum,
    PageBuild,
    Public,
    RepositoryDispatch(Box<RepositoryDispatchPayload>),
    Status,
    Watch,
    WorkflowCall(Box<WorkflowCallPayload>),
    WorkflowRun,
    Custom {
        name: String,
        payload: serde_json::Value,
    },
}

/// Trait for event types that can produce a JSON payload.
pub trait EventPayload {
    /// The event type name (e.g. "push", "pull_request").
    fn event_name(&self) -> &str;

    /// Serializes the event payload to a JSON value.
    fn to_payload(&self) -> serde_json::Value;
}

/// Payload for `push` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PushPayload {
    pub r#ref: String,
    pub before: String,
    pub after: String,
    pub repository: RepositoryInfo,
    pub pusher: UserInfo,
    pub sender: UserInfo,
    pub created: bool,
    pub deleted: bool,
    pub forced: bool,
    pub commits: Vec<CommitInfo>,
    pub head_commit: Option<CommitInfo>,
    pub compare: String,
}

/// Payload for `pull_request` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PullRequestPayload {
    pub action: String,
    pub number: u64,
    pub pull_request: PullRequestInfo,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}

/// Payload for `workflow_dispatch` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WorkflowDispatchPayload {
    pub inputs: HashMap<String, String>,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
    pub workflow: String,
    pub r#ref: String,
}

/// Payload for `release` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ReleasePayload {
    pub action: String,
    pub release: ReleaseInfo,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}

/// Payload for `issues` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct IssuesPayload {
    pub action: String,
    pub issue: IssueInfo,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}

/// Payload for `issue_comment` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct IssueCommentPayload {
    pub action: String,
    pub issue: IssueInfo,
    pub comment: CommentInfo,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}

/// Payload for `create` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CreatePayload {
    pub ref_type: String,
    pub r#ref: String,
    pub master_branch: String,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}

/// Payload for `delete` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DeletePayload {
    pub ref_type: String,
    pub r#ref: String,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}

/// Payload for `fork` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ForkPayload {
    pub forkee: RepositoryInfo,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}

/// Payload for `repository_dispatch` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RepositoryDispatchPayload {
    pub action: String,
    pub client_payload: serde_json::Value,
    pub repository: RepositoryInfo,
    pub sender: UserInfo,
}

/// Payload for `workflow_call` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WorkflowCallPayload {
    pub inputs: HashMap<String, serde_json::Value>,
    pub secrets: HashMap<String, String>,
}

/// Repository information included in event payloads.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RepositoryInfo {
    pub name: String,
    pub full_name: String,
    pub owner: UserInfo,
    pub private: bool,
    pub html_url: String,
    pub default_branch: String,
    pub clone_url: String,
    pub ssh_url: String,
}

/// User/actor information included in event payloads.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct UserInfo {
    pub name: String,
    pub email: String,
    pub login: String,
}

/// Commit information for push events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub timestamp: String,
    pub author: UserInfo,
    pub committer: UserInfo,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub modified: Vec<String>,
}

/// Pull request information.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PullRequestInfo {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub head: BranchRef,
    pub base: BranchRef,
    pub user: UserInfo,
    pub html_url: String,
    pub draft: bool,
    pub merged: bool,
    pub mergeable: Option<bool>,
}

/// Branch reference for pull requests.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct BranchRef {
    pub r#ref: String,
    pub sha: String,
    pub repo: RepositoryInfo,
    pub label: String,
}

/// Release information.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub draft: bool,
    pub prerelease: bool,
    pub html_url: String,
}

/// Issue information.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct IssueInfo {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub user: UserInfo,
    pub labels: Vec<LabelInfo>,
    pub html_url: String,
}

/// Comment information.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct CommentInfo {
    pub id: u64,
    pub body: String,
    pub user: UserInfo,
    pub html_url: String,
}

/// Label information.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LabelInfo {
    pub name: String,
    pub color: String,
}

impl EventPayload for Event {
    fn event_name(&self) -> &str {
        match self {
            Event::Push(_) => "push",
            Event::PullRequest(_) => "pull_request",
            Event::WorkflowDispatch(_) => "workflow_dispatch",
            Event::Schedule => "schedule",
            Event::Release(_) => "release",
            Event::Issues(_) => "issues",
            Event::IssueComment(_) => "issue_comment",
            Event::Create(_) => "create",
            Event::Delete(_) => "delete",
            Event::Fork(_) => "fork",
            Event::Gollum => "gollum",
            Event::PageBuild => "page_build",
            Event::Public => "public",
            Event::RepositoryDispatch(_) => "repository_dispatch",
            Event::Status => "status",
            Event::Watch => "watch",
            Event::WorkflowCall(_) => "workflow_call",
            Event::WorkflowRun => "workflow_run",
            Event::Custom { name, .. } => name.as_str(),
        }
    }

    fn to_payload(&self) -> serde_json::Value {
        match self {
            Event::Push(p) => serde_json::to_value(p).unwrap_or_default(),
            Event::PullRequest(p) => serde_json::to_value(p).unwrap_or_default(),
            Event::WorkflowDispatch(p) => serde_json::to_value(p).unwrap_or_default(),
            Event::Schedule => serde_json::json!({}),
            Event::Release(p) => serde_json::to_value(p).unwrap_or_default(),
            Event::Issues(p) => serde_json::to_value(p).unwrap_or_default(),
            Event::IssueComment(p) => serde_json::to_value(p).unwrap_or_default(),
            Event::Create(p) => serde_json::to_value(p).unwrap_or_default(),
            Event::Delete(p) => serde_json::to_value(p).unwrap_or_default(),
            Event::Fork(p) => serde_json::to_value(p).unwrap_or_default(),
            Event::Gollum => serde_json::json!({}),
            Event::PageBuild => serde_json::json!({}),
            Event::Public => serde_json::json!({}),
            Event::RepositoryDispatch(p) => serde_json::to_value(p).unwrap_or_default(),
            Event::Status => serde_json::json!({}),
            Event::Watch => serde_json::json!({}),
            Event::WorkflowCall(p) => serde_json::to_value(p).unwrap_or_default(),
            Event::WorkflowRun => serde_json::json!({}),
            Event::Custom { payload, .. } => payload.clone(),
        }
    }
}

impl Event {
    /// Creates a push event with sensible defaults for local execution.
    pub fn push_default(branch: &str, repo: &RepositoryInfo) -> Self {
        Event::Push(Box::new(PushPayload {
            r#ref: format!("refs/heads/{}", branch),
            before: "0000000000000000000000000000000000000000".to_owned(),
            after: "0000000000000000000000000000000000000000".to_owned(),
            repository: repo.clone(),
            pusher: UserInfo {
                name: "act".to_owned(),
                email: "act@localhost".to_owned(),
                login: "act".to_owned(),
            },
            sender: UserInfo {
                name: "act".to_owned(),
                email: "act@localhost".to_owned(),
                login: "act".to_owned(),
            },
            created: false,
            deleted: false,
            forced: false,
            commits: vec![],
            head_commit: None,
            compare: String::new(),
        }))
    }

    /// Creates a pull_request event with sensible defaults for local execution.
    pub fn pull_request_default(number: u64, repo: &RepositoryInfo) -> Self {
        Event::PullRequest(Box::new(PullRequestPayload {
            action: "opened".to_owned(),
            number,
            pull_request: PullRequestInfo {
                number,
                title: "Local PR".to_owned(),
                body: None,
                head: BranchRef {
                    r#ref: "refs/heads/feature".to_owned(),
                    sha: "0000000000000000000000000000000000000000".to_owned(),
                    repo: repo.clone(),
                    label: "feature".to_owned(),
                },
                base: BranchRef {
                    r#ref: "refs/heads/main".to_owned(),
                    sha: "0000000000000000000000000000000000000000".to_owned(),
                    repo: repo.clone(),
                    label: "main".to_owned(),
                },
                user: UserInfo {
                    name: "act".to_owned(),
                    email: "act@localhost".to_owned(),
                    login: "act".to_owned(),
                },
                html_url: String::new(),
                draft: false,
                merged: false,
                mergeable: None,
            },
            repository: repo.clone(),
            sender: UserInfo {
                name: "act".to_owned(),
                email: "act@localhost".to_owned(),
                login: "act".to_owned(),
            },
        }))
    }

    /// Creates a workflow_dispatch event with the given inputs.
    pub fn workflow_dispatch(inputs: HashMap<String, String>, repo: &RepositoryInfo) -> Self {
        Event::WorkflowDispatch(Box::new(WorkflowDispatchPayload {
            inputs,
            repository: repo.clone(),
            sender: UserInfo {
                name: "act".to_owned(),
                email: "act@localhost".to_owned(),
                login: "act".to_owned(),
            },
            workflow: String::new(),
            r#ref: "refs/heads/main".to_owned(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_repo() -> RepositoryInfo {
        RepositoryInfo {
            name: "test-repo".to_owned(),
            full_name: "owner/test-repo".to_owned(),
            owner: UserInfo {
                name: "owner".to_owned(),
                email: "owner@example.com".to_owned(),
                login: "owner".to_owned(),
            },
            private: false,
            html_url: "https://github.com/owner/test-repo".to_owned(),
            default_branch: "main".to_owned(),
            clone_url: "https://github.com/owner/test-repo.git".to_owned(),
            ssh_url: "git@github.com:owner/test-repo.git".to_owned(),
        }
    }

    #[test]
    fn push_event_name() {
        let event = Event::push_default("main", &test_repo());
        assert_eq!(event.event_name(), "push");
    }

    #[test]
    fn push_event_payload_is_valid_json() {
        let event = Event::push_default("main", &test_repo());
        let payload = event.to_payload();
        let json_str = serde_json::to_string(&payload).unwrap();
        assert!(json_str.contains("refs/heads/main"));
        assert!(json_str.contains("test-repo"));
    }

    #[test]
    fn pull_request_event_name() {
        let event = Event::pull_request_default(42, &test_repo());
        assert_eq!(event.event_name(), "pull_request");
    }

    #[test]
    fn workflow_dispatch_event_name() {
        let mut inputs = HashMap::new();
        inputs.insert("name".to_owned(), "world".to_owned());
        let event = Event::workflow_dispatch(inputs, &test_repo());
        assert_eq!(event.event_name(), "workflow_dispatch");
    }

    #[test]
    fn schedule_event_has_empty_payload() {
        let event = Event::Schedule;
        assert_eq!(event.event_name(), "schedule");
        assert_eq!(event.to_payload(), serde_json::json!({}));
    }

    #[test]
    fn custom_event() {
        let event = Event::Custom {
            name: "deployment".to_owned(),
            payload: serde_json::json!({"environment": "production"}),
        };
        assert_eq!(event.event_name(), "deployment");
        assert_eq!(
            event.to_payload(),
            serde_json::json!({"environment": "production"})
        );
    }
}
