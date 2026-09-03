use crate::application::ports::outbound::ContainerPort;

/// Request DTO for the
/// [`ResolveNodeBinaryPort`](crate::application::ports::inbound::resolve_node_binary_port::ResolveNodeBinaryPort)
/// inbound port.
pub struct ResolveNodeBinaryRequest<'a> {
    /// Container the JavaScript action will run in.
    pub container: &'a dyn ContainerPort,
}
