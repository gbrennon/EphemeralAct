/// Response DTO for the
/// [`ListWorkflowsPort`](crate::core::ports::inbound::list_workflows_port::ListWorkflowsPort)
/// inbound port.
///
/// Carries a raw summary of each workflow discovered in the repository without
/// exposing the core [`Workflow`](crate::core::workflow::Workflow) entity.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ListWorkflowsResponse {
    /// Raw summaries of the discovered workflows.
    pub workflows: Vec<WorkflowListItem>,
}

impl ListWorkflowsResponse {
    /// Creates a new list-workflows response.
    pub fn new(workflows: Vec<WorkflowListItem>) -> Self {
        Self { workflows }
    }
}

/// Raw summary of a single discovered workflow.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkflowListItem {
    /// The display name of the workflow, if any.
    pub name: Option<String>,
    /// The name of the workflow file.
    pub file: Option<String>,
}

impl WorkflowListItem {
    /// Creates a new workflow list item.
    pub fn new(name: Option<String>, file: Option<String>) -> Self {
        Self { name, file }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_response_empty_by_default() {
        let response = ListWorkflowsResponse::new(vec![]);
        assert!(response.workflows.is_empty());
    }

    #[test]
    fn new_workflow_list_item_keeps_fields() {
        let item = WorkflowListItem::new(Some("ci".into()), Some("ci.yml".into()));
        assert_eq!(item.name.as_deref(), Some("ci"));
        assert_eq!(item.file.as_deref(), Some("ci.yml"));
    }

    #[test]
    fn new_workflow_list_item_allows_missing_fields() {
        let item = WorkflowListItem::new(None, None);
        assert_eq!(item.name, None);
        assert_eq!(item.file, None);
    }
}
