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

    struct FakeContainer;

    impl ContainerPort for FakeContainer {
        fn exec(
            &self,
            _cmd: &[String],
            _workdir: Option<&str>,
            _env: &HashMap<String, String>,
        ) -> Result<ExecResult, ContainerError> {
            Ok(ExecResult {
                exit_code: 0,
                stdout: String::new(),
                stderr: String::new(),
            })
        }
        fn copy_to(&self, _p: &str, _e: &[FileEntry]) -> Result<(), ContainerError> {
            Ok(())
        }
        fn copy_from(&self, _p: &str) -> Result<Vec<FileEntry>, ContainerError> {
            Ok(vec![])
        }
        fn remove(&self) -> Result<(), ContainerError> {
            Ok(())
        }
        fn get_runner_context(&self) -> Result<RunnerContext, ContainerError> {
            Ok(RunnerContext::default())
        }
    }

    impl ContainerRuntimePort for FakeRuntime {
        fn pull_image(&self, _i: &str, _p: Option<&str>) -> Result<(), ContainerError> {
            Ok(())
        }
        fn create_container(
            &self,
            _c: &ContainerConfig,
        ) -> Result<Box<dyn ContainerPort>, ContainerError> {
            Ok(Box::new(FakeContainer))
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
            Ok(HostInfo {
                os: "linux".into(),
                arch: "amd64".into(),
                engine_version: "1.0".into(),
            })
        }
    }

    #[test]
    fn fake_supports_all_runtime_operations() {
        let (runtime, _, _) = FakeRuntime::new();
        runtime.pull_image("img", None).unwrap();
        runtime.get_host_info().unwrap();
        let container = runtime
            .create_container(&ContainerConfig {
                image: "img".into(),
                platform: None,
                env: HashMap::new(),
                binds: vec![],
                workdir: None,
                cmd: None,
                entrypoint: None,
                network: None,
                name: None,
                runner_context: RunnerContext::default(),
            })
            .unwrap();
        container.exec(&[], None, &HashMap::new()).unwrap();
        container.copy_to("p", &[]).unwrap();
        assert!(container.copy_from("p").unwrap().is_empty());
        container.remove().unwrap();
        container.get_runner_context().unwrap();
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
