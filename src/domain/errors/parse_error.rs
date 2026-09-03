/// Error returned when parsing fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Human-readable description of what went wrong.
    pub message: String,
    /// Approximate position in the input where the error occurred.
    pub position: usize,
}
