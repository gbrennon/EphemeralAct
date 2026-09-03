use std::{collections::BTreeSet, fs, path::Path};

use crate::application::{
    dtos::{ListActionsRequest, ListActionsResponse},
    ports::{
        inbound::list_actions_port::ListActionsPort,
        outbound::workflow_file_parser::WorkflowFileParserPort,
    },
};

/// Service that lists action references (`uses:`) across repository workflow files.
///
/// Scans `.github/workflows/` and `.forgejo/workflows/` directories under the
/// requested repository path, parses each workflow file using the injected
/// `WorkflowFileParserPort`, collects all `uses:` action references, and returns
/// them as a deduplicated sorted list.
pub struct ListActionsService {
    /// Parser for extracting action references from workflow YAML content.
    parser: Box<dyn WorkflowFileParserPort>,
}

impl ListActionsService {
    /// Create a new service with the given parser.
    pub fn new(parser: Box<dyn WorkflowFileParserPort>) -> Self {
        Self { parser }
    }
}

impl ListActionsPort for ListActionsService {
    fn execute(
        &self,
        request: ListActionsRequest,
    ) -> Result<ListActionsResponse, Box<dyn std::error::Error>> {
        let path = request.path;

        let workflow_dirs = vec![
            Path::new(".github/workflows"),
            Path::new(".forgejo/workflows"),
        ];

        let mut actions = BTreeSet::new();

        for workflow_dir in &workflow_dirs {
            let full_path = path.join(workflow_dir);
            if !full_path.exists() {
                continue;
            }

            let entries = fs::read_dir(&full_path)?;
            for entry in entries.flatten() {
                let file_path = entry.path();
                if file_path
                    .extension()
                    .is_some_and(|ext| ext == "yaml" || ext == "yml")
                    && let Ok(content) = fs::read_to_string(&file_path)
                {
                    let extracted = self.parser.extract_actions(&content);
                    for action in extracted {
                        actions.insert(action);
                    }
                }
            }
        }

        Ok(ListActionsResponse::new(actions.into_iter().collect()))
    }
}
