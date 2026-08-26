use super::application::Application;
use crate::{
    core::ports::inbound::{
        list_actions_port::ListActionsPort, list_workflows_port::ListWorkflowsPort,
        run_act_port::RunActPort,
    },
    presentation::cli::Cli,
};

/// Builds presentation-layer objects from infrastructure dependencies.
pub struct CompositionRoot;

impl CompositionRoot {
    /// Assembles the presentation layer from fully-wired ports and
    /// returns an [`Application`].
    pub fn compose(
        run_port: Box<dyn RunActPort>,
        list_workflows_port: Box<dyn ListWorkflowsPort>,
        list_actions_port: Box<dyn ListActionsPort>,
    ) -> Application {
        Application {
            cli: Cli::new(run_port, list_workflows_port, list_actions_port),
        }
    }
}
