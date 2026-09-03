use serde::Deserialize;

use super::step::Step;

/// The execution strategy for an action.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(tag = "using")]
pub enum ActionRuns {
    /// Composite action: runs shell steps in the job's container.
    #[serde(rename = "composite")]
    Composite {
        /// Steps to execute sequentially.
        steps: Vec<Step>,
    },

    /// Node action: runs a JavaScript file (node12 variant).
    #[serde(rename = "node12")]
    Node12 {
        /// Entry point script.
        main: String,
    },

    /// Node action: runs a JavaScript file (node16 variant).
    #[serde(rename = "node16")]
    Node16 {
        /// Entry point script.
        main: String,
    },

    /// Node action (node20 variant).
    #[serde(rename = "node20")]
    Node20 {
        /// Entry point script.
        main: String,
    },

    /// Docker action: runs a container image (not yet executed).
    #[serde(rename = "docker")]
    Docker {
        /// Docker image to run.
        image: String,
    },
}
