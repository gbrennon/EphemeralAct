use crate::application::dtos::ResolveNodeBinaryRequest;

/// Inbound port for finding the node interpreter inside a container.
pub trait ResolveNodeBinaryPort: Send + Sync {
    /// Returns the interpreter to run a JavaScript action with.
    fn execute(&self, request: ResolveNodeBinaryRequest<'_>) -> String;
}
