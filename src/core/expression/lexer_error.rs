/// Errors that can occur during lexing.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LexerError {
    /// An unexpected character was encountered at the given byte position.
    #[error("unexpected character '{0}' at position {1}")]
    UnexpectedChar(char, usize),
    /// A single-quoted string was not terminated before end of input.
    #[error("unterminated string starting at position {0}")]
    UnterminatedString(usize),
}
