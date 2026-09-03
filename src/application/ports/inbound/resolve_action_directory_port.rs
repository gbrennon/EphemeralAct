use crate::{
    application::dtos::{ResolveActionDirectoryRequest, ResolvedActionDirectory},
    domain::errors::StepError,
};

/// Inbound port for deciding where the action a step references lives.
pub trait ResolveActionDirectoryPort {
    /// Classifies the reference and resolves it to a directory.
    fn execute(
        &self,
        request: ResolveActionDirectoryRequest<'_>,
    ) -> Result<ResolvedActionDirectory, StepError>;
}
