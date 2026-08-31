use super::list_actions_args::ListActionsArgs;
use crate::core::{dtos::ListActionsResponse, ports::inbound::list_actions_port::ListActionsPort};

/// Handles the `list-actions` subcommand by dispatching parsed CLI arguments to the
/// application port.
pub struct ListActionsHandler;

impl ListActionsHandler {
    /// Executes the `list-actions` subcommand: converts CLI args to domain objects,
    /// calls the application port, renders the response to stdout.
    pub fn handle(
        args: ListActionsArgs,
        port: &dyn ListActionsPort,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let request = args.to_domain()?;
        let response = port.execute(request)?;
        Self::render(&response);
        Ok(())
    }

    /// Renders the action list as plain text for the terminal.
    fn render(response: &ListActionsResponse) {
        for action in &response.actions {
            println!("{action}");
        }
    }
}
