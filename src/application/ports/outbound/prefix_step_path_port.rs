use std::collections::HashMap;

use crate::application::dtos::PrefixStepPathRequest;

/// Inbound port for prefixing a step's `PATH` with directories earlier steps
/// exported.
pub trait PrefixStepPathPort {
    /// Returns the environment with its `PATH` prefixed by the additions.
    fn execute(&self, request: PrefixStepPathRequest<'_>) -> HashMap<String, String>;
}
