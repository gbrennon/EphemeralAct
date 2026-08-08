/// Errors that can occur during workflow planning.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum PlanError {
    /// A circular dependency was detected.
    #[error(
        "circular dependency detected: '{job}' depends on '{dependency}' which creates a cycle"
    )]
    CycleDetected { job: String, dependency: String },

    /// A job depends on a job that doesn't exist.
    #[error("job '{job}' depends on '{dependency}' which is not defined in the workflow")]
    MissingDependency { job: String, dependency: String },

    /// Dependencies could not be fully resolved (should not happen after cycle detection).
    #[error("unresolved dependencies remain after topological sort")]
    UnresolvedDependencies,
}
