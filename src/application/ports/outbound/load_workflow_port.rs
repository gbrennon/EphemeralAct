use crate::{application::dtos::LoadWorkflowRequest, domain::workflow::Workflow};

pub trait LoadWorkflowPort: Send + Sync {
    fn execute(
        &self,
        request: LoadWorkflowRequest<'_>,
    ) -> Result<Workflow, Box<dyn std::error::Error>>;
}
