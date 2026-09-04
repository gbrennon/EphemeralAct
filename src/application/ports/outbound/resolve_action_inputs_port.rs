use std::collections::HashMap;

use crate::application::dtos::ResolveActionInputsRequest;

/// Inbound port for resolving the inputs an action runs with.
pub trait ResolveActionInputsPort: Send + Sync {
    /// Overlays the step's `with:` values on the action's declared defaults.
    fn execute(&self, request: ResolveActionInputsRequest<'_>) -> HashMap<String, String>;
}
