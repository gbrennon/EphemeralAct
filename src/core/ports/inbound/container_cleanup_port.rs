/// Inbound port for handling act-run-completed events.
///
/// Implemented by application services that react to workflow completion
/// (e.g. stopping containers, sending notifications).
pub trait ContainerCleanupUseCase {
    /// Handle the completion of a workflow run.
    ///
    /// `container_names` lists every container created during the run.
    /// Implementations SHOULD stop and remove these containers but MUST
    /// NOT delete cached images so the user does not have to re-download
    /// them on the next run.
    fn handle_act_run_completed(&self, container_names: &[String]);
}
