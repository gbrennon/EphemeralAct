use crate::core::ports::inbound::{
    list_actions_port::ListActionsPort, list_workflows_port::ListWorkflowsPort,
    run_act_port::RunActPort,
};

/// Holds all three application ports for the presentation layer.
pub struct AppContainer {
    pub run_act_port: Box<dyn RunActPort>,
    pub list_workflows_port: Box<dyn ListWorkflowsPort>,
    pub list_actions_port: Box<dyn ListActionsPort>,
}
