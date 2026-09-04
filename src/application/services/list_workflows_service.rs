use std::error::Error;

use crate::application::{
    dtos::{ListWorkflowsRequest, ListWorkflowsResponse},
    ports::{inbound::list_workflows_port::ListWorkflowsPort, outbound::WorkflowSourcePort},
};

/// Application service implementing the `ListWorkflowsPort` entrypoint.
///
/// Agnostic by construction: it states the intent ("list the workflows of this
/// repository") and delegates every storage and parsing concern to the outbound
/// [`WorkflowSourcePort`].
pub struct ListWorkflowsService {
    workflow_source: Box<dyn WorkflowSourcePort>,
}

impl ListWorkflowsService {
    pub fn new(workflow_source: Box<dyn WorkflowSourcePort>) -> Self {
        Self { workflow_source }
    }
}

impl ListWorkflowsPort for ListWorkflowsService {
    fn execute(
        &self,
        request: ListWorkflowsRequest,
    ) -> Result<ListWorkflowsResponse, Box<dyn Error>> {
        let workflows = self.workflow_source.list_workflows(&request.repository)?;
        Ok(ListWorkflowsResponse::new(workflows))
    }
}
