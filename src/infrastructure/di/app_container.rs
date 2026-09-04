use crate::application::ports::inbound::{
    list_actions_port::ListActionsPort, list_workflows_port::ListWorkflowsPort,
    run_action_port::RunActionPort, run_all_workflows_port::RunAllWorkflowsPort,
    run_workflow_port::RunWorkflowPort,
};

pub struct AppContainer {
    pub run_all_workflows_port: Box<dyn RunAllWorkflowsPort>,
    pub run_workflow_port: Box<dyn RunWorkflowPort>,
    pub run_action_port: Box<dyn RunActionPort>,
    pub list_workflows_port: Box<dyn ListWorkflowsPort>,
    pub list_actions_port: Box<dyn ListActionsPort>,
}
