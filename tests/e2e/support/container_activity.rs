use parking_lot::Mutex;
use std::{collections::HashMap, sync::Arc};

/// Records everything the application asks a container runtime to do, so an
/// end-to-end test can assert on the commands a workflow produced without
/// starting a real container.
///
/// Every clone observes the same recording, which lets the same instance be
/// handed to a runtime and to the containers it creates.
#[derive(Clone, Default)]
pub struct ContainerActivity {
    commands: Arc<Mutex<Vec<Vec<String>>>>,
    environments: Arc<Mutex<Vec<HashMap<String, String>>>>,
    copied_paths: Arc<Mutex<Vec<String>>>,
    pulled_images: Arc<Mutex<Vec<String>>>,
    stopped_containers: Arc<Mutex<Vec<String>>>,
}

impl ContainerActivity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_command(&self, command: &[String], environment: &HashMap<String, String>) {
        self.commands.lock().push(command.to_vec());
        self.environments.lock().push(environment.clone());
    }

    pub fn record_copy(&self, container_path: &str) {
        self.copied_paths.lock().push(container_path.into());
    }

    pub fn record_pulled_image(&self, image: &str) {
        self.pulled_images.lock().push(image.into());
    }

    pub fn record_stopped_container(&self, name: &str) {
        self.stopped_containers.lock().push(name.into());
    }

    /// Returns the script of every executed command, which for a shell step is
    /// the payload of `bash -c`.
    pub fn scripts(&self) -> Vec<String> {
        self.commands
            .lock()
            .iter()
            .filter_map(|command| command.last().cloned())
            .collect()
    }

    pub fn ran_script(&self, script: &str) -> bool {
        self.scripts().iter().any(|executed| executed == script)
    }

    pub fn ran_command_containing(&self, fragment: &str) -> bool {
        self.commands
            .lock()
            .iter()
            .any(|command| command.iter().any(|argument| argument.contains(fragment)))
    }

    pub fn ran_command_with_environment(&self, name: &str, value: &str) -> bool {
        self.environments
            .lock()
            .iter()
            .any(|environment| environment.get(name).is_some_and(|found| found == value))
    }

    /// Reports whether `first` was executed before `second`, and false when
    /// either script never ran.
    pub fn ran_before(&self, first: &str, second: &str) -> bool {
        let scripts = self.scripts();
        let first_position = scripts.iter().position(|script| script == first);
        let second_position = scripts.iter().position(|script| script == second);
        match (first_position, second_position) {
            (Some(first_index), Some(second_index)) => first_index < second_index,
            _ => false,
        }
    }

    pub fn copied_to_path_containing(&self, fragment: &str) -> bool {
        self.copied_paths
            .lock()
            .iter()
            .any(|path| path.contains(fragment))
    }

    pub fn pulled_images(&self) -> Vec<String> {
        self.pulled_images.lock().clone()
    }

    pub fn stopped_containers(&self) -> Vec<String> {
        self.stopped_containers.lock().clone()
    }
}
