/// Trait for event types that can produce a JSON payload.
pub trait EventPayload {
    /// The event type name (e.g. "push", "pull_request").
    fn event_name(&self) -> &str;

    /// Serializes the event payload to a JSON value.
    fn to_payload(&self) -> serde_json::Value;
}
