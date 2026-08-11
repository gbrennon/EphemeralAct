use super::run::Run;

/// A stage is a group of runs that execute in parallel.
///
/// Stages are separated by dependency boundaries: all runs in a stage
/// must complete before the next stage begins.
#[derive(Debug, Clone, PartialEq)]
pub struct Stage {
    /// The runs in this stage (execute in parallel).
    pub runs: Vec<Run>,
}
