use serde::Deserialize;

/// Concurrency configuration to limit parallel workflow runs.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Concurrency {
    /// The concurrency group name.
    pub group: String,

    /// Whether to cancel in-progress runs in the same group.
    #[serde(rename = "cancel-in-progress")]
    pub cancel_in_progress: Option<bool>,
}
