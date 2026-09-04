#[cfg(test)]
mod tests {
    use ephact::presentation::composition_root::CompositionRoot;

    use crate::common::fakes::{
        fake_list_actions_port::FakeListActionsPort,
        fake_list_workflows_port::FakeListWorkflowsPort,
        fake_run_all_workflows_port::FakeRunAllWorkflowsPort,
        fake_run_workflow_port::FakeRunWorkflowPort,
    };

    #[test]
    fn compose_creates_app_with_fake_port() {
        let run_workflow_port = Box::new(FakeRunWorkflowPort::new(true));
        let run_all_workflows_port = Box::new(FakeRunAllWorkflowsPort::new(true));
        let list_workflows_port = Box::new(FakeListWorkflowsPort::new());
        let list_actions_port = Box::new(FakeListActionsPort::new());
        let _app = CompositionRoot::compose(
            run_workflow_port,
            run_all_workflows_port,
            list_workflows_port,
            list_actions_port,
        );
    }
}
