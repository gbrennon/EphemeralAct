use crate::presentation::cli::Cli;

/// Fully-wired presentation layer, returned by [`CompositionRoot::compose`].
///
/// Each field is a concrete presentation object assembled from the
/// dependency graph built in the infrastructure layer.
pub struct Application {
    pub cli: Cli,
}
