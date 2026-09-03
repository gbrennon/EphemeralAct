/// Payload for [`DomainEvent::ActRunCompleted`].
///
/// [`DomainEvent::ActRunCompleted`]: super::domain_event::DomainEvent::ActRunCompleted
#[derive(Debug, Clone)]
pub struct ActRunCompletedPayload {
    /// Names of containers created during the run. Handlers use these to
    /// stop and remove containers without deleting cached images.
    pub container_names: Vec<String>,
    /// Whether the workflow succeeded.
    pub success: bool,
}
