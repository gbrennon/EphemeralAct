use serde::Deserialize;

/// A declared input parameter for an action.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ActionInput {
    /// Description of the input.
    #[serde(default)]
    pub description: Option<String>,

    /// Whether the input is required.
    #[serde(default)]
    pub required: bool,

    /// Default value if not provided.
    #[serde(default)]
    pub default: Option<String>,
}
