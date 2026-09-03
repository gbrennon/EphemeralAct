use crate::domain::{ActRunConfig, Repository};

/// Request DTO for the
/// [`BuildRunContextPort`](crate::application::ports::outbound::build_run_context_port::BuildRunContextPort)
/// inbound port.
pub struct BuildRunContextRequest<'a> {
    /// Configuration the run was started with.
    pub config: &'a ActRunConfig,
    /// Repository the run executes against.
    pub repository: &'a Repository,
}
