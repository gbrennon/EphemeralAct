use crate::application::dtos::ReadStepPathExportsRequest;

/// Inbound port for reading the `PATH` additions a step exported.
pub trait ReadStepPathExportsPort {
    /// Returns the directories the step exported, or none when it exported nothing.
    fn execute(&self, request: ReadStepPathExportsRequest<'_>) -> Vec<String>;
}
