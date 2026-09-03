use std::collections::HashMap;

use serde::Deserialize;

/// A matrix defining variable combinations for job expansion.
#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct Matrix {
    /// The variables and their possible values.
    #[serde(flatten)]
    pub variables: HashMap<String, Vec<serde_yaml::Value>>,

    /// Additional combinations to include.
    #[serde(default)]
    pub include: Vec<HashMap<String, serde_yaml::Value>>,

    /// Combinations to exclude from the matrix.
    #[serde(default)]
    pub exclude: Vec<HashMap<String, serde_yaml::Value>>,
}
