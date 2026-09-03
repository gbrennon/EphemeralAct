use super::stage::Stage;

/// A plan is the complete execution order for a workflow.
///
/// Stages execute sequentially; runs within a stage execute in parallel.
#[derive(Debug, Clone, PartialEq)]
pub struct Plan {
    /// The ordered stages of execution.
    pub stages: Vec<Stage>,
}
