pub mod act_run_config;
pub mod entities;
pub mod errors;
pub mod ports;
pub mod services;
pub mod shared_types;
pub mod value_objects;

// Domain re-exports
// Application-layer re-exports
pub use ports::{inbound::run_act_port::RunActUseCase, outbound::act_executor_port::ActExecutor};
pub use services::run_act_service::RunActService;
pub use shared_types::ExecutionResult;

pub use self::{
    act_run_config::ActRunConfig,
    entities::{
        ephemeral_repository::{EphemeralRepository, TempDirTemplate},
        repository::Repository,
    },
    errors::CoreError,
    value_objects::{
        ActEvent, ActExtraArg, ActInput, ActJob, ActWorkflow, CleanupPolicy, ContainerEngine,
        GitDirKind, RepoPath, RepositoryName, Secret,
    },
};
