use std::collections::HashMap;

use serde::Deserialize;

use super::WorkflowDispatchInput;

/// Configuration for a specific event type.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct EventConfig {
    /// Branch filters (glob patterns).
    #[serde(default)]
    pub branches: Vec<String>,

    /// Branch-ignore filters (glob patterns).
    #[serde(rename = "branches-ignore")]
    #[serde(default)]
    pub branches_ignore: Vec<String>,

    /// Tag filters (glob patterns).
    #[serde(default)]
    pub tags: Vec<String>,

    /// Tag-ignore filters (glob patterns).
    #[serde(rename = "tags-ignore")]
    #[serde(default)]
    pub tags_ignore: Vec<String>,

    /// Path filters (glob patterns).
    #[serde(default)]
    pub paths: Vec<String>,

    /// Path-ignore filters (glob patterns).
    #[serde(rename = "paths-ignore")]
    #[serde(default)]
    pub paths_ignore: Vec<String>,

    /// Activity types for events that support them (e.g. `issues`, `pull_request`).
    #[serde(default)]
    pub types: Vec<String>,

    /// Input definitions for `workflow_dispatch`.
    #[serde(default)]
    pub inputs: HashMap<String, WorkflowDispatchInput>,

    /// Cron schedule for `schedule` events.
    #[serde(default)]
    pub cron: Vec<String>,
}
