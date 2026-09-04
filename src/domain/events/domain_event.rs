/// Domain events published by application services.
///
/// Events state a fact about something that already happened. They are
/// dispatched in-memory through the application's event bus outbound port, and
/// infrastructure handlers subscribe to the variants they react to (e.g.
/// container cleanup once a run completed).
#[derive(Debug, Clone)]
pub enum DomainEvent {
    /// Published when a workflow run completes (success or failure).
    ActRunCompleted(super::act_run_completed_payload::ActRunCompletedPayload),
}
