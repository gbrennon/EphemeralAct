#![allow(dead_code)]
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use ephemeral_act::core::ports::outbound::{
    ContainerConfig, ContainerError, ContainerPort, ContainerRuntimePort, ExecResult, HostInfo,
};

use super::fake_container_handle::FakeContainerHandle;

/// Container runtime that hands out containers replaying queued exec results
/// and recording every command and file copy they receive.
pub struct FakeRuntime {
    pub pulled_images: RefCell<Vec<String>>,
    pub created_containers: RefCell<Vec<ContainerConfig>>,
    pub exec_results: Rc<RefCell<Vec<ExecResult>>>,
    pub executed_commands: Rc<RefCell<Vec<Vec<String>>>>,
    pub exec_environments: Rc<RefCell<Vec<HashMap<String, String>>>>,
    pub copied_paths: Rc<RefCell<Vec<String>>>,
    pub removed_containers: RefCell<Vec<String>>,
    pub stopped_containers: RefCell<Vec<String>>,
}

impl FakeRuntime {
    pub fn new() -> Self {
        Self {
            pulled_images: RefCell::new(vec![]),
            created_containers: RefCell::new(vec![]),
            exec_results: Rc::new(RefCell::new(vec![])),
            executed_commands: Rc::new(RefCell::new(vec![])),
            exec_environments: Rc::new(RefCell::new(vec![])),
            copied_paths: Rc::new(RefCell::new(vec![])),
            removed_containers: RefCell::new(vec![]),
            stopped_containers: RefCell::new(vec![]),
        }
    }

    /// Returns the scripts passed to `bash -c`, in execution order.
    pub fn executed_scripts(&self) -> Vec<String> {
        self.executed_commands
            .borrow()
            .iter()
            .filter(|command| command.len() == 3 && command[1] == "-c")
            .map(|command| command[2].clone())
            .collect()
    }
}

impl ContainerRuntimePort for FakeRuntime {
    fn pull_image(&self, image: &str, _platform: Option<&str>) -> Result<(), ContainerError> {
        self.pulled_images.borrow_mut().push(image.to_string());
        Ok(())
    }

    fn create_container(
        &self,
        config: &ContainerConfig,
    ) -> Result<Box<dyn ContainerPort>, ContainerError> {
        self.created_containers.borrow_mut().push(config.clone());
        Ok(Box::new(FakeContainerHandle::new(
            self.exec_results.clone(),
            self.executed_commands.clone(),
            self.exec_environments.clone(),
            self.copied_paths.clone(),
        )))
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
        Ok(HostInfo {
            os: "linux".into(),
            arch: "amd64".into(),
            engine_version: "1.0".into(),
        })
    }
}
