/// Error types for the core domain.
///
/// This module contains error types used across the application's core logic,
/// including configuration errors (`CoreError`), runtime step execution errors (`StepError`),
/// expression evaluation and parsing errors (`EvalError`, `ParseError`, `LexerError`),
/// workflow planning errors (`PlanError`), and container operation errors (`ContainerError`).
pub mod action_error;
pub mod container_error;
pub mod core_error;
pub mod eval_error;
pub mod lexer_error;
pub mod parse_error;
pub mod plan_error;
pub mod step_error;

pub use action_error::ActionError;
pub use container_error::ContainerError;
pub use core_error::CoreError;
pub use eval_error::EvalError;
pub use lexer_error::LexerError;
pub use parse_error::ParseError;
pub use plan_error::PlanError;
pub use step_error::StepError;
