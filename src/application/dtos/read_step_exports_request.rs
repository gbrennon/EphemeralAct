use crate::application::ports::outbound::ContainerPort;

/// Request DTO for the
/// [`ReadStepExportsPort`](crate::application::ports::outbound::read_step_exports_port::ReadStepExportsPort)
/// inbound port.
pub struct ReadStepExportsRequest<'a> {
    /// Container the step just ran in.
    pub container: &'a dyn ContainerPort,
}
