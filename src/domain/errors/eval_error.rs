/// Errors that can occur during expression function evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    /// A function received an argument of the wrong type.
    TypeError(String),

    /// A function received the wrong number of arguments.
    ArgCount(String),

    /// A format string could not be parsed or applied.
    FormatError(String),

    /// A JSON value could not be parsed or serialized.
    JsonError(String),
}
