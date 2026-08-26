#![allow(dead_code)]
use ephemeral_act::core::ports::outbound::workflow_file_parser::WorkflowFileParserPort;

/// Fake implementation of `WorkflowFileParserPort` for testing.
///
/// Parses YAML content to extract workflow summaries and action references
/// using simple string matching (not a full YAML parser).
pub struct FakeWorkflowFileParser;

impl FakeWorkflowFileParser {
    pub fn new() -> Self {
        Self
    }
}

impl WorkflowFileParserPort for FakeWorkflowFileParser {
    fn extract_actions(&self, yaml: &str) -> Vec<String> {
        let mut actions = Vec::new();
        for line in yaml.lines() {
            let trimmed = line.trim();
            // Handle both `- uses:` and `uses:` patterns
            let rest = if let Some(r) = trimmed.strip_prefix("- uses:") {
                r
            } else if let Some(r) = trimmed.strip_prefix("uses:") {
                r
            } else {
                continue;
            };
            let action_ref = rest.trim();
            if !action_ref.is_empty() && !action_ref.starts_with('#') {
                actions.push(action_ref.to_string());
            }
        }
        actions
    }

    fn extract_summary(&self, yaml: &str) -> (Option<String>, String) {
        let mut name = None;
        for line in yaml.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("name:") {
                let n = rest.trim().trim_matches(|c| c == '"' || c == '\'');
                if !n.is_empty() {
                    name = Some(n.to_string());
                    break;
                }
            }
        }
        (name, "workflow.yml".to_string())
    }
}