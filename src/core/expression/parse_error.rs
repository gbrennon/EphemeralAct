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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_formats_message_and_position() {
        let err = ParseError {
            message: "unexpected token".into(),
            position: 42,
        };
        assert_eq!(err.to_string(), "parse error at position 42: unexpected token");
    }

    #[test]
    fn error_trait_implemented() {
        let err = ParseError {
            message: "test".into(),
            position: 0,
        };
        let _: &dyn std::error::Error = &err;
    }
}
