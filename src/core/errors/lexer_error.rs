/// Errors that can occur during lexing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexerError {
    /// An unexpected character was encountered at the given byte position.
    UnexpectedChar(char, usize),
    /// A single-quoted string was not terminated before end of input.
    UnterminatedString(usize),
}
