/// Error returned when parsing fails.
#[derive(Debug, PartialEq)]
pub struct ParseError {
    /// Human-readable description of what went wrong.
    pub message: String,
    /// Approximate position in the input where the error occurred.
    pub position: usize,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "parse error at position {}: {}",
            self.position, self.message
        )
    }
}

impl std::error::Error for ParseError {}
