/// Outbound port for parsing CI workflow files.
///
/// Implementors discover and parse workflow YAML files from a repository path,
/// extracting action references and workflow summaries.
pub trait WorkflowFileParserPort: Send + Sync {
    /// Extract action references (`uses:`) from a workflow YAML string.
    fn extract_actions(&self, yaml: &str) -> Vec<String>;

    /// Extract workflow name and filename summary from a workflow YAML string.
    fn extract_summary(&self, yaml: &str) -> (Option<String>, String);
}
