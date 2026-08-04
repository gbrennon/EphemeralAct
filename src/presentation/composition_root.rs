use crate::core::ports::inbound::run_act_port::RunActUseCase;
use crate::presentation::cli::Cli;

/// Fully-wired presentation layer, returned by [`CompositionRoot::compose`].
///
/// Each field is a concrete presentation object assembled from the
/// dependency graph built in the infrastructure layer.
pub struct Application {
    pub cli: Cli,
}

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
