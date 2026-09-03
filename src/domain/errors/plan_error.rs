/// Errors that can occur during workflow planning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanError {
    /// A circular dependency was detected.
    CycleDetected { job: String, dependency: String },

    /// A job depends on a job that doesn't exist.
    MissingDependency { job: String, dependency: String },

    /// Dependencies could not be fully resolved (should not happen after cycle detection).
    UnresolvedDependencies,
}
