use crate::core::dtos::ContainerCleanupRequest;

/// Inbound port for handling act-run-completed events.
///
/// Implemented by application services that react to workflow completion
/// (e.g. stopping containers, sending notifications).
pub trait ContainerCleanupPort {
    /// Handles the completion of a workflow run.
    ///
    /// Implementations SHOULD stop and remove the containers listed in the
    /// request but MUST NOT delete cached images so the user does not have
    /// to re-download them on the next run.
    fn execute(&self, request: ContainerCleanupRequest);
}
