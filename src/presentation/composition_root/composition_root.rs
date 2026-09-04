use super::application::Application;
use crate::{
    application::ports::inbound::{
        list_actions_port::ListActionsPort, list_workflows_port::ListWorkflowsPort,
        run_all_workflows_port::RunAllWorkflowsPort, run_workflow_port::RunWorkflowPort,
    },
    presentation::cli::Cli,
};

pub struct CompositionRoot;

impl CompositionRoot {
    pub fn compose(
        run_workflow_port: Box<dyn RunWorkflowPort>,
        run_all_workflows_port: Box<dyn RunAllWorkflowsPort>,
        list_workflows_port: Box<dyn ListWorkflowsPort>,
        list_actions_port: Box<dyn ListActionsPort>,
    ) -> Application {
        Application {
            cli: Cli::new(
                run_workflow_port,
                run_all_workflows_port,
                list_workflows_port,
                list_actions_port,
            ),
        }
    }
}
