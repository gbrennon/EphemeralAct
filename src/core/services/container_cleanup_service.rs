use crate::core::ports::{
    inbound::container_cleanup_port::ContainerCleanupUseCase,
    outbound::container_runtime::ContainerRuntime,
};

/// Application service that reacts to workflow completion by cleaning up
/// containers created during the run.
///
/// Implements [`ContainerCleanupUseCase`] — stops and removes containers
/// but does NOT delete cached images.
pub struct ContainerCleanupService<R: ContainerRuntime> {
    runtime: R,
}

impl<R: ContainerRuntime> ContainerCleanupService<R> {
    pub fn new(runtime: R) -> Self {
        Self { runtime }
    }
}

impl<R: ContainerRuntime> ContainerCleanupUseCase for ContainerCleanupService<R> {
    fn handle_act_run_completed(&self, container_names: &[String]) {
        for name in container_names {
            let _ = self.runtime.stop_container(name);
            eprintln!("Container stopped: {name}");
            let _ = self.runtime.remove_container(name);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    use super::*;
    use crate::core::ports::outbound::{
        Container, ContainerConfig, ContainerError, ContainerRuntime, ExecResult, FileEntry,
        HostInfo, RunnerContext,
    };

    type Log = Rc<RefCell<Vec<String>>>;

    struct FakeRuntime {
        stopped: Log,
        removed: Log,
    }

    impl FakeRuntime {
        fn new() -> (Self, Log, Log) {
            let stopped = Rc::new(RefCell::new(Vec::new()));
            let removed = Rc::new(RefCell::new(Vec::new()));
            (
                Self {
                    stopped: stopped.clone(),
                    removed: removed.clone(),
                },
                stopped,
                removed,
            )
        }
    }

    #[allow(dead_code)]
    struct FakeContainer;

    impl Container for FakeContainer {
        fn exec(
            &self,
            _cmd: &[String],
            _workdir: Option<&str>,
            _env: &HashMap<String, String>,
        ) -> Result<ExecResult, ContainerError> {
            unimplemented!()
        }
        fn copy_to(&self, _p: &str, _e: &[FileEntry]) -> Result<(), ContainerError> {
            unimplemented!()
        }
        fn copy_from(&self, _p: &str) -> Result<Vec<FileEntry>, ContainerError> {
            unimplemented!()
        }
        fn remove(&self) -> Result<(), ContainerError> {
            unimplemented!()
        }
        fn get_runner_context(&self) -> Result<RunnerContext, ContainerError> {
            unimplemented!()
        }
    }

    impl ContainerRuntime for FakeRuntime {
        fn pull_image(&self, _i: &str, _p: Option<&str>) -> Result<(), ContainerError> {
            unimplemented!()
        }
        fn create_container(
            &self,
            _c: &ContainerConfig,
        ) -> Result<Box<dyn Container>, ContainerError> {
            unimplemented!()
        }
        fn remove_container(&self, name: &str) -> Result<(), ContainerError> {
            self.removed.borrow_mut().push(name.to_string());
            Ok(())
        }
        fn stop_container(&self, name: &str) -> Result<(), ContainerError> {
            self.stopped.borrow_mut().push(name.to_string());
            Ok(())
        }
        fn get_host_info(&self) -> Result<HostInfo, ContainerError> {
            unimplemented!()
        }
    }

    #[test]
    fn new_creates_service() {
        let (runtime, _, _) = FakeRuntime::new();
        let _service = ContainerCleanupService::new(runtime);
    }

    #[test]
    fn handle_empty_list_does_nothing() {
        let (runtime, stopped, removed) = FakeRuntime::new();
        let service = ContainerCleanupService::new(runtime);
        service.handle_act_run_completed(&[]);
        assert!(stopped.borrow().is_empty());
        assert!(removed.borrow().is_empty());
    }

    #[test]
    fn handle_stops_and_removes_each_container() {
        let (runtime, stopped, removed) = FakeRuntime::new();
        let service = ContainerCleanupService::new(runtime);
        let names: Vec<String> = vec!["app1".into(), "app2".into()];
        service.handle_act_run_completed(&names);
        assert_eq!(*stopped.borrow(), vec!["app1", "app2"]);
        assert_eq!(*removed.borrow(), vec!["app1", "app2"]);
    }
}
