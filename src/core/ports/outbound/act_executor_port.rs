use crate::core::{ActRunConfig, Repository, shared_types::ExecutionResult};

/// Outbound port for executing CI workflows.
///
/// Adapters receive the full configuration and repository and are responsible
/// for translating the domain objects into platform-specific CLI invocations.
pub trait ActExecutor {
    fn execute_act(
        &self,
        config: &ActRunConfig,
        repository: &Repository,
    ) -> Result<ExecutionResult, String>;
}
