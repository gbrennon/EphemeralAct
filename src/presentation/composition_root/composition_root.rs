use super::application::Application;
use crate::{core::ports::inbound::run_act_port::RunActUseCase, presentation::cli::Cli};

/// Builds presentation-layer objects from infrastructure dependencies.
pub struct CompositionRoot;

impl CompositionRoot {
    /// Assembles the presentation layer from a fully-wired use case and
    /// returns an [`Application`].
    pub fn compose(use_case: impl RunActUseCase + 'static) -> Application {
        Application {
            cli: Cli::new(use_case),
        }
    }
}
