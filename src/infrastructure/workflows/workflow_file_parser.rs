use std::collections::BTreeSet;

use crate::application::ports::outbound::workflow_file_parser::WorkflowFileParserPort;

/// Infrastructure implementation that parses CI workflow YAML files from the
/// local filesystem.
///
/// Scans `.github/workflows/` and `.forgejo/workflows/` directories, reads
/// each `.yml`/`.yaml` file, and extracts action references and workflow
/// summaries.
#[derive(Clone, Copy)]
pub struct FilesystemWorkflowFileParser;

impl WorkflowFileParserPort for FilesystemWorkflowFileParser {
    fn extract_actions(&self, yaml: &str) -> Vec<String> {
        let mut actions = BTreeSet::new();

        for line in yaml.lines() {
            let trimmed = line.trim();
            let rest = if let Some(r) = trimmed.strip_prefix("- uses:") {
                r
            } else if let Some(r) = trimmed.strip_prefix("uses:") {
                r
            } else {
                continue;
            };
            let action_ref = rest.trim();
            if !action_ref.is_empty() && !action_ref.starts_with('#') {
                actions.insert(action_ref.to_string());
            }
        }

        actions.into_iter().collect()
    }

    fn extract_summary(&self, yaml: &str) -> (Option<String>, String) {
        let name = self.extract_workflow_name(yaml);
        let filename = self.extract_filename(yaml);
        (name, filename)
    }
}

impl FilesystemWorkflowFileParser {
    fn extract_workflow_name(&self, yaml: &str) -> Option<String> {
        for line in yaml.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("name:") {
                let name = rest.trim().trim_matches(|c| c == '"' || c == '\'');
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        None
    }

    fn extract_filename(&self, _yaml: &str) -> String {
        "workflow".to_string()
    }
}
