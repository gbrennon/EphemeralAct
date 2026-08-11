use serde::Deserialize;

/// An input parameter for `workflow_dispatch` events.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct WorkflowDispatchInput {
    /// Description of the input.
    pub description: Option<String>,

    /// Whether the input is required.
    #[serde(default)]
    pub required: bool,

    /// Default value for the input.
    #[serde(default)]
    pub default: Option<String>,

    /// The type of the input (string, choice, boolean, environment).
    #[serde(rename = "type")]
    #[serde(default)]
    pub input_type: Option<String>,

    /// Available options for `choice` type inputs.
    #[serde(default)]
    pub options: Vec<String>,
}
