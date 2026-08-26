use super::list_workflows_args::ListWorkflowsArgs;
use crate::core::{
    dtos::ListWorkflowsResponse,
    ports::inbound::list_workflows_port::ListWorkflowsPort,
};

/// Handles the `list-workflows` subcommand by dispatching parsed CLI arguments to the
/// application port.
pub struct ListWorkflowsHandler;

impl ListWorkflowsHandler {
    /// Executes the `list-workflows` subcommand: converts CLI args to domain objects,
    /// calls the application port, renders the response to stdout.
    pub fn handle(
        args: ListWorkflowsArgs,
        port: &dyn ListWorkflowsPort,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = args.to_domain()?;
        let response = port.execute(request)?;
        Self::render(&response);
        Ok(())
    }

    /// Renders the workflow list as plain text for the terminal.
    fn render(response: &ListWorkflowsResponse) {
        for wf in &response.workflows {
            let file = wf.file.as_deref().unwrap_or("?");
            let name = wf.name.as_deref().unwrap_or("");
            if name.is_empty() {
                println!("{file}");
            } else {
                println!("{file} ({name})");
            }
        }
    }
}