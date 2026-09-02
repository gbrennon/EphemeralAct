use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// Records everything the application asks a container runtime to do, so an
/// end-to-end test can assert on the commands a workflow produced without
/// starting a real container.
///
/// Every clone observes the same recording, which lets the same instance be
/// handed to a runtime and to the containers it creates.
#[derive(Clone, Default)]
pub struct ContainerActivity {
    commands: Rc<RefCell<Vec<Vec<String>>>>,
    environments: Rc<RefCell<Vec<HashMap<String, String>>>>,
    copied_paths: Rc<RefCell<Vec<String>>>,
    pulled_images: Rc<RefCell<Vec<String>>>,
    stopped_containers: Rc<RefCell<Vec<String>>>,
}

impl ContainerActivity {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record_command(&self, command: &[String], environment: &HashMap<String, String>) {
        self.commands.borrow_mut().push(command.to_vec());
        self.environments.borrow_mut().push(environment.clone());
    }

    pub fn record_copy(&self, container_path: &str) {
        self.copied_paths.borrow_mut().push(container_path.into());
    }

    pub fn record_pulled_image(&self, image: &str) {
        self.pulled_images.borrow_mut().push(image.into());
    }

    pub fn record_stopped_container(&self, name: &str) {
        self.stopped_containers.borrow_mut().push(name.into());
    }

    /// Returns the script of every executed command, which for a shell step is
    /// the payload of `bash -c`.
    pub fn scripts(&self) -> Vec<String> {
        self.commands
            .borrow()
            .iter()
            .filter_map(|command| command.last().cloned())
            .collect()
    }

    pub fn ran_script(&self, script: &str) -> bool {
        self.scripts().iter().any(|executed| executed == script)
    }

    pub fn ran_command_containing(&self, fragment: &str) -> bool {
        self.commands
            .borrow()
            .iter()
            .any(|command| command.iter().any(|argument| argument.contains(fragment)))
    }

    pub fn ran_command_with_environment(&self, name: &str, value: &str) -> bool {
        self.environments
            .borrow()
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
            .borrow()
            .iter()
            .any(|path| path.contains(fragment))
    }

    pub fn pulled_images(&self) -> Vec<String> {
        self.pulled_images.borrow().clone()
    }

    pub fn stopped_containers(&self) -> Vec<String> {
        self.stopped_containers.borrow().clone()
    }
}
