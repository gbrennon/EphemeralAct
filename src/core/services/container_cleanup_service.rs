use crate::core::{
    dtos::ContainerCleanupRequest,
    ports::{
        inbound::container_cleanup_port::ContainerCleanupPort,
        outbound::container_runtime::ContainerRuntimePort,
    },
};

/// Application service that reacts to workflow completion by cleaning up
/// containers created during the run.
///
/// Implements [`ContainerCleanupPort`] — stops and removes containers
/// but does NOT delete cached images.
pub struct ContainerCleanupService<R: ContainerRuntimePort> {
    runtime: R,
}

impl<R: ContainerRuntimePort> ContainerCleanupService<R> {
    pub fn new(runtime: R) -> Self {
        Self { runtime }
    }
}

impl<R: ContainerRuntimePort> ContainerCleanupPort for ContainerCleanupService<R> {
    fn execute(&self, request: ContainerCleanupRequest) {
        for name in &request.container_names {
            let _ = self.runtime.stop_container(name);
            let _ = self.runtime.remove_container(name);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, collections::HashMap, rc::Rc};

    use super::*;
    use crate::core::ports::outbound::{
        ContainerConfig, ContainerError, ContainerPort, ContainerRuntimePort, ExecResult,
        FileEntry, HostInfo, RunnerContext,
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

    impl ContainerPort for FakeContainer {
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

    impl ContainerRuntimePort for FakeRuntime {
        fn pull_image(&self, _i: &str, _p: Option<&str>) -> Result<(), ContainerError> {
            unimplemented!()
        }
        fn create_container(
            &self,
            _c: &ContainerConfig,
        ) -> Result<Box<dyn ContainerPort>, ContainerError> {
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
    fn execute_empty_request_does_nothing() {
        let (runtime, stopped, removed) = FakeRuntime::new();
        let service = ContainerCleanupService::new(runtime);
        service.execute(ContainerCleanupRequest::default());
        assert!(stopped.borrow().is_empty());
        assert!(removed.borrow().is_empty());
    }

    #[test]
    fn execute_stops_and_removes_each_container() {
        let (runtime, stopped, removed) = FakeRuntime::new();
        let service = ContainerCleanupService::new(runtime);
        let request = ContainerCleanupRequest::new(vec!["app1".into(), "app2".into()]);
        service.execute(request);
        assert_eq!(*stopped.borrow(), vec!["app1", "app2"]);
        assert_eq!(*removed.borrow(), vec!["app1", "app2"]);
    }
}
