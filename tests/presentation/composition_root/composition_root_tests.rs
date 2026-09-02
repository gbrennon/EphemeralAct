#[cfg(test)]
mod tests {
    use ephact::presentation::composition_root::CompositionRoot;

    use crate::common::fakes::{
        fake_list_actions_port::FakeListActionsPort,
        fake_list_workflows_port::FakeListWorkflowsPort, fake_run_act_port::FakeRunActPort,
    };

    #[test]
    fn compose_creates_app_with_fake_port() {
        let run_port = Box::new(FakeRunActPort::new(true));
        let list_workflows_port = Box::new(FakeListWorkflowsPort::new());
        let list_actions_port = Box::new(FakeListActionsPort::new());
        let _app = CompositionRoot::compose(run_port, list_workflows_port, list_actions_port);
    }
}
