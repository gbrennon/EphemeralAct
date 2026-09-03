use std::collections::HashMap;

use serde::Serialize;

/// Payload for `workflow_call` events.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct WorkflowCallPayload {
    pub inputs: HashMap<String, serde_json::Value>,
    pub secrets: HashMap<String, String>,
}
