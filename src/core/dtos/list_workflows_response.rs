use super::workflow_list_item::WorkflowListItem;

/// Result of listing the workflow files of a repository.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ListWorkflowsResponse {
    /// Workflows found in the repository, in discovery order.
    pub workflows: Vec<WorkflowListItem>,
}

impl ListWorkflowsResponse {
    /// Creates a response carrying the discovered workflows.
    pub fn new(workflows: Vec<WorkflowListItem>) -> Self {
        Self { workflows }
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
    fn new_response_keeps_the_given_workflows() {
        let response =
            ListWorkflowsResponse::new(vec![WorkflowListItem::new(None, Some("ci.yml".into()))]);

        assert_eq!(
            response.workflows,
            vec![WorkflowListItem::new(None, Some("ci.yml".into()))]
        );
    }
}
