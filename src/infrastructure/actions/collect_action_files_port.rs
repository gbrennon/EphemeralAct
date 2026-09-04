use crate::{
    application::dtos::{CollectActionFilesRequest, CollectActionFilesResponse},
    domain::errors::StepError,
};

/// Inbound port for reading the files that make up an action.
pub trait CollectActionFilesPort: Send + Sync {
    /// Walks the action directory and reads every file it holds.
    fn execute(
        &self,
        request: CollectActionFilesRequest<'_>,
    ) -> Result<CollectActionFilesResponse, StepError>;
}
