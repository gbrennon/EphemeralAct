use std::collections::HashMap;

use crate::domain::workflow::Job;

/// A single job run within a stage.
///
/// When a job has a matrix strategy, it expands into multiple `Run` instances,
/// one per matrix combination.
#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    /// The workflow this run belongs to.
    pub workflow_name: Option<String>,

    /// The job ID (key in the workflow's `jobs` map).
    pub job_id: String,

    /// The job definition.
    pub job: Job,

    /// Matrix combination values, if this run is part of a matrix expansion.
    pub matrix_values: Option<HashMap<String, String>>,
}
