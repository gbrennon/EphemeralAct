#![allow(dead_code)]
use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};

use ephact::application::dtos::ExecResult;
use ephact::application::ports::outbound::container_port::ContainerPort;
use ephact::domain::errors::ContainerError;
use ephact::infrastructure::containers::ContainerConfig;
use ephact::infrastructure::containers::ContainerRuntimePort;
use ephact::infrastructure::containers::HostInfo;

use super::fake_container_handle::FakeContainerHandle;

/// Container runtime that hands out containers replaying queued exec results
/// and recording every command and file copy they receive.
pub struct FakeRuntime {
    pub pulled_images: Mutex<Vec<String>>,
    pub created_containers: Mutex<Vec<ContainerConfig>>,
    pub exec_results: Arc<Mutex<Vec<ExecResult>>>,
    pub executed_commands: Arc<Mutex<Vec<Vec<String>>>>,
    pub exec_environments: Arc<Mutex<Vec<HashMap<String, String>>>>,
    pub copied_paths: Arc<Mutex<Vec<String>>>,
    pub removed_containers: Mutex<Vec<String>>,
    pub stopped_containers: Mutex<Vec<String>>,
}

impl FakeRuntime {
    pub fn new() -> Self {
        Self {
            pulled_images: Mutex::new(vec![]),
            created_containers: Mutex::new(vec![]),
            exec_results: Arc::new(Mutex::new(vec![])),
            executed_commands: Arc::new(Mutex::new(vec![])),
            exec_environments: Arc::new(Mutex::new(vec![])),
            copied_paths: Arc::new(Mutex::new(vec![])),
            removed_containers: Mutex::new(vec![]),
            stopped_containers: Mutex::new(vec![]),
        }
    }

    /// Returns the scripts passed to `bash -c`, in execution order.
    pub fn executed_scripts(&self) -> Vec<String> {
        self.executed_commands
            .lock()
            .iter()
            .filter(|command| command.len() == 3 && command[1] == "-c")
            .map(|command| command[2].clone())
            .collect()
    }
}

impl ContainerRuntimePort for FakeRuntime {
    fn pull_image(&self, image: &str, _platform: Option<&str>) -> Result<(), ContainerError> {
        self.pulled_images.lock().push(image.to_string());
        Ok(())
    }

    fn create_container(
        &self,
        config: &ContainerConfig,
    ) -> Result<Box<dyn ContainerPort>, ContainerError> {
        self.created_containers.lock().push(config.clone());
        Ok(Box::new(FakeContainerHandle::new(
            self.exec_results.clone(),
            self.executed_commands.clone(),
            self.exec_environments.clone(),
            self.copied_paths.clone(),
        )))
    }

    fn remove_container(&self, name: &str) -> Result<(), ContainerError> {
        self.removed_containers.lock().push(name.to_string());
        Ok(())
    }

    fn stop_container(&self, name: &str) -> Result<(), ContainerError> {
        self.stopped_containers.lock().push(name.to_string());
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
