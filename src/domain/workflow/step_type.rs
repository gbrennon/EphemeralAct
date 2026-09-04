/// How a workflow step declares the work it performs.
#[derive(Debug, Clone, PartialEq)]
pub enum StepType {
    Run,
    Uses,
    Composite,
    Invalid,
}
