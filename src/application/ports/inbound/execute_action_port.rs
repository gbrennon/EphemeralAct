use crate::{
    application::dtos::{ExecuteActionRequest, ExecuteActionResponse},
    domain::errors::StepError,
};

/// Inbound port for executing a single action referenced by a workflow step.
///
/// Implementations resolve the `uses:` reference - a path inside the
/// repository, or a repository on any forge that has to be fetched first - and
/// run the resulting action definition inside the container supplied with the
/// request.
///
/// The response reports the action's exit status and output; a [`StepError`] is
/// returned only when the action could not be run at all, and it carries any
/// output produced before the failure.
pub trait ExecuteActionPort {
    /// Resolves and runs the requested action.
    ///
    /// # Errors
    ///
    /// Returns [`StepError`] when the reference cannot be resolved, the action
    /// definition cannot be read, or the container refuses to run it.
    fn execute(&self, request: ExecuteActionRequest) -> Result<ExecuteActionResponse, StepError>;
}
