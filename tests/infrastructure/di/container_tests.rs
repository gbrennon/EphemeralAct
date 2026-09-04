use std::sync::Arc;

use ephact::{
    application::ports::inbound::{
        list_actions_port::ListActionsPort, list_workflows_port::ListWorkflowsPort,
        run_action_port::RunActionPort, run_all_workflows_port::RunAllWorkflowsPort,
        run_workflow_port::RunWorkflowPort,
    },
    infrastructure::{AppContainer, Container},
};

use crate::common::fakes::{
    fake_action_fetcher::FakeActionFetcher, fake_image_mapper::FakeImageMapper,
    fake_runtime::FakeRuntime, fake_workflow_source::FakeWorkflowSource,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_returns_app_container() {
        let runtime = Arc::new(FakeRuntime::new());
        let _container = Container::with_runtime(runtime);
    }

    #[test]
    fn build_result_contains_all_ports() {
        let runtime = Arc::new(FakeRuntime::new());
        let workflow_source = Arc::new(FakeWorkflowSource::new());
        let container: AppContainer = Container::with_collaborators(
            runtime,
            Box::new(FakeImageMapper),
            Box::new(FakeActionFetcher::returning(std::path::PathBuf::new())),
            workflow_source,
        );
        fn _assert_run_all_workflows(_: Box<dyn RunAllWorkflowsPort>) {}
        fn _assert_run_workflow(_: Box<dyn RunWorkflowPort>) {}
        fn _assert_run_action(_: Box<dyn RunActionPort>) {}
        fn _assert_list_workflows(_: Box<dyn ListWorkflowsPort>) {}
        fn _assert_list_actions(_: Box<dyn ListActionsPort>) {}
        _assert_run_all_workflows(container.run_all_workflows_port);
        _assert_run_workflow(container.run_workflow_port);
        _assert_run_action(container.run_action_port);
        _assert_list_workflows(container.list_workflows_port);
        _assert_list_actions(container.list_actions_port);
    }
}
