use std::cell::RefCell;
use std::collections::HashMap;

use ephemeral_act::core::{
    events::DomainEvent,
    ports::outbound::{
        Container, ContainerConfig, ContainerError, ContainerRuntime, EventPublisher,
        ExecResult, FileEntry, HostInfo, ImageMapper, RunnerContext,
    },
};

pub struct FakeRuntime {
    pub pulled_images: RefCell<Vec<String>>,
    pub created_containers: RefCell<Vec<ContainerConfig>>,
    pub exec_results: RefCell<Vec<ExecResult>>,
    pub removed_containers: RefCell<Vec<String>>,
    pub stopped_containers: RefCell<Vec<String>>,
}

impl FakeRuntime {
    pub fn new() -> Self {
        Self {
            pulled_images: RefCell::new(vec![]),
            created_containers: RefCell::new(vec![]),
            exec_results: RefCell::new(vec![]),
            removed_containers: RefCell::new(vec![]),
            stopped_containers: RefCell::new(vec![]),
        }
    }
}

impl ContainerRuntime for FakeRuntime {
    fn pull_image(&self, image: &str, _platform: Option<&str>) -> Result<(), ContainerError> {
        self.pulled_images.borrow_mut().push(image.to_string());
        Ok(())
    }

    fn create_container(&self, config: &ContainerConfig) -> Result<Box<dyn Container>, ContainerError> {
        self.created_containers.borrow_mut().push(config.clone());
        Ok(Box::new(FakeContainerHandle { exec_results: self.exec_results.clone() }))
    }

    fn remove_container(&self, name: &str) -> Result<(), ContainerError> {
        self.removed_containers.borrow_mut().push(name.to_string());
        Ok(())
    }

    fn stop_container(&self, name: &str) -> Result<(), ContainerError> {
        self.stopped_containers.borrow_mut().push(name.to_string());
        Ok(())
    }

    fn get_host_info(&self) -> Result<HostInfo, ContainerError> {
        Ok(HostInfo { os: "linux".into(), arch: "amd64".into(), engine_version: "1.0".into() })
    }
}

struct FakeContainerHandle {
    exec_results: RefCell<Vec<ExecResult>>,
}

impl Container for FakeContainerHandle {
    fn exec(&self, _cmd: &[String], _workdir: Option<&str>, _env: &HashMap<String, String>) -> Result<ExecResult, ContainerError> {
        Ok(self.exec_results.borrow_mut().pop().unwrap_or(ExecResult {
            exit_code: 0,
            stdout: String::new(),
            stderr: String::new(),
        }))
    }

    fn copy_to(&self, _path: &str, _entries: &[FileEntry]) -> Result<(), ContainerError> { Ok(()) }
    fn copy_from(&self, _path: &str) -> Result<Vec<FileEntry>, ContainerError> { Ok(vec![]) }
    fn remove(&self) -> Result<(), ContainerError> { Ok(()) }

    fn get_runner_context(&self) -> Result<RunnerContext, ContainerError> {
        Ok(RunnerContext {
            workspace: "/github/workspace".into(),
            home: "/root".into(),
            action_path: "/actions".into(),
            temp: "/tmp".into(),
            tool_cache: "/tool-cache".into(),
            env: HashMap::new(),
        })
    }
}

pub struct FakeEventPublisher(RefCell<Vec<DomainEvent>>);

impl FakeEventPublisher {
    pub fn new() -> Self { Self(RefCell::new(Vec::new())) }
}

impl EventPublisher for FakeEventPublisher {
    fn publish(&self, event: DomainEvent) { self.0.borrow_mut().push(event); }
}

pub struct FakeImageMapper;

impl ImageMapper for FakeImageMapper {
    fn map(&self, platform: &str) -> String { platform.to_string() }
    fn fallback(&self) -> String { "fake-image:latest".into() }
}
