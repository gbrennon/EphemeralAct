use thiserror::Error;

/// Errors that can occur during expression function evaluation.
#[derive(Error, Debug, PartialEq)]
pub enum EvalError {
    /// A function received an argument of the wrong type.
    #[error("type error: {0}")]
    TypeError(String),

    /// A function received the wrong number of arguments.
    #[error("argument count error: {0}")]
    ArgCount(String),

    /// A format string could not be parsed or applied.
    #[error("format error: {0}")]
    FormatError(String),

    /// A JSON value could not be parsed or serialized.
    #[error("JSON error: {0}")]
    JsonError(String),
}
