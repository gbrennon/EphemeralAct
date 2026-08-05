use crate::{
    core::{ports::inbound::run_act_port::RunActUseCase, services::run_act_service::RunActService},
    infrastructure::actions_executor::ActionsExecutor,
};

/// Dependency-injection container that constructs and wires all application
/// dependencies. Returns a fully-wired [`RunActUseCase`] ready for the
/// presentation layer to consume.
pub struct Container;

impl Container {
    /// Builds the application service graph and returns the entry-point
    /// use case.
    pub fn build() -> impl RunActUseCase {
        let executor = ActionsExecutor::new();
        RunActService::new(executor)
    }
}
