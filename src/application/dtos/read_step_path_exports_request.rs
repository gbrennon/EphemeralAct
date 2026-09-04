use crate::application::ports::outbound::container_port::ContainerPort;

/// Request DTO for the
/// [`ReadStepPathExportsPort`](crate::application::ports::inbound::read_step_path_exports_port::ReadStepPathExportsPort)
/// inbound port.
pub struct ReadStepPathExportsRequest<'a> {
    /// Container the step just ran in.
    pub container: &'a dyn ContainerPort,
}
