pub mod act_run_config;
pub mod entities;
pub mod errors;
pub mod ports;
pub mod services;
pub mod shared_types;
pub mod value_objects;

// Domain re-exports
pub use self::act_run_config::ActRunConfig;
pub use self::entities::ephemeral_repository::{EphemeralRepository, TempDirTemplate};
pub use self::entities::repository::Repository;
pub use self::errors::CoreError;
pub use self::value_objects::{
    ActEvent, ActExtraArg, ActInput, ActJob, ActWorkflow, CleanupPolicy, ContainerEngine,
    GitDirKind, RepoPath, RepositoryName, Secret,
};
pub use shared_types::ExecutionResult;

// Application-layer re-exports
pub use ports::inbound::run_act_port::RunActUseCase;
pub use ports::outbound::act_executor_port::ActExecutor;
pub use services::run_act_service::RunActService;
