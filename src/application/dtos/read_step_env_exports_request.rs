use crate::application::ports::outbound::container_port::ContainerPort;

/// Request DTO for the
/// [`ReadStepEnvExportsPort`](crate::application::ports::inbound::read_step_env_exports_port::ReadStepEnvExportsPort)
/// inbound port.
pub struct ReadStepEnvExportsRequest<'a> {
    /// Container the step just ran in.
    pub container: &'a dyn ContainerPort,
}
