use std::error::Error;

use crate::application::{
    dtos::{ListActionsRequest, ListActionsResponse},
    ports::{inbound::list_actions_port::ListActionsPort, outbound::WorkflowSourcePort},
};

/// Application service implementing the `ListActionsPort` entrypoint.
///
/// Agnostic by construction: it states the intent ("list the actions of this
/// repository") and delegates every storage and parsing concern to the outbound
/// [`WorkflowSourcePort`].
pub struct ListActionsService {
    workflow_source: Box<dyn WorkflowSourcePort>,
}

impl ListActionsService {
    pub fn new(workflow_source: Box<dyn WorkflowSourcePort>) -> Self {
        Self { workflow_source }
    }
}

impl ListActionsPort for ListActionsService {
    fn execute(&self, request: ListActionsRequest) -> Result<ListActionsResponse, Box<dyn Error>> {
        let actions = self.workflow_source.list_actions(&request.repository)?;
        Ok(ListActionsResponse::new(actions))
    }
}
