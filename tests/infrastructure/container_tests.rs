use ephemeral_act::{
    core::ports::inbound::{
        list_actions_port::ListActionsPort,
        list_workflows_port::ListWorkflowsPort,
        run_act_port::RunActPort,
    },
    infrastructure::{AppContainer, Container, runners::ContainerRuntimeAdapter},
};

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! require_container_runtime {
        () => {
            if ContainerRuntimeAdapter::detect().is_err() {
                eprintln!("SKIP: no container runtime available (Docker or Podman required)");
                return;
            }
        };
    }

    #[test]
    fn build_returns_app_container_when_runtime_available() {
        require_container_runtime!();
        let _container = Container::build();
    }

    #[test]
    fn build_result_contains_all_ports() {
        require_container_runtime!();
        let container: AppContainer = Container::build();
        fn _assert_run(_: Box<dyn RunActPort>) {}
        fn _assert_list_workflows(_: Box<dyn ListWorkflowsPort>) {}
        fn _assert_list_actions(_: Box<dyn ListActionsPort>) {}
        _assert_run(container.run_act_port);
        _assert_list_workflows(container.list_workflows_port);
        _assert_list_actions(container.list_actions_port);
    }
}
