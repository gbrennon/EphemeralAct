use std::{fs, path::Path};

use crate::application::{
    dtos::{ListWorkflowsRequest, ListWorkflowsResponse, WorkflowListItem},
    ports::{
        inbound::list_workflows_port::ListWorkflowsPort,
        outbound::workflow_file_parser::WorkflowFileParserPort,
    },
};

/// Service that lists CI workflow files found in a repository.
///
/// Discovers workflow files (`.github/workflows/` and `.forgejo/workflows/`)
/// under the requested repository path, parses each one using the injected
/// `WorkflowFileParserPort` for a display name and filename, and returns
/// raw summaries in `ListWorkflowsResponse`.
pub struct ListWorkflowsService {
    /// Parser for extracting workflow summaries from YAML content.
    parser: Box<dyn WorkflowFileParserPort>,
}

impl ListWorkflowsService {
    /// Create a new service with the given parser.
    pub fn new(parser: Box<dyn WorkflowFileParserPort>) -> Self {
        Self { parser }
    }
}

impl ListWorkflowsPort for ListWorkflowsService {
    fn execute(
        &self,
        request: ListWorkflowsRequest,
    ) -> Result<ListWorkflowsResponse, Box<dyn std::error::Error>> {
        let path = request.path;

        let workflow_dirs = vec![
            Path::new(".github/workflows"),
            Path::new(".forgejo/workflows"),
        ];

        let mut workflows = Vec::new();

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
                    let (name, file) = self.parser.extract_summary(&content);
                    workflows.push(WorkflowListItem::new(name, Some(file)));
                }
            }
        }

        Ok(ListWorkflowsResponse::new(workflows))
    }
}
