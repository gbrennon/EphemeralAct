use std::{collections::BTreeSet, error::Error, fs};

use crate::{
    application::ports::outbound::WorkflowSourcePort, domain::entities::repository::Repository,
};

/// Infrastructure adapter that reads workflow definitions from the filesystem.
///
/// This adapter is the concrete implementation of `WorkflowSourcePort` and lives
/// entirely in the infrastructure layer. It knows nothing about application
/// services or ports - it simply reads workflow files and returns their contents.
pub struct FilesystemWorkflowSource {
    workflow_dirs: Vec<String>,
}

impl FilesystemWorkflowSource {
    pub fn new(workflow_dirs: &[&str]) -> Self {
        Self {
            workflow_dirs: workflow_dirs.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn find_workflow_files(
        &self,
        repo: &Repository,
    ) -> Result<Vec<std::path::PathBuf>, Box<dyn Error>> {
        let repo_path = repo.path().as_path();
        let mut workflows = Vec::new();

        for dir in &self.workflow_dirs {
            let workflows_dir = repo_path.join(dir);
            if workflows_dir.exists() {
                for entry in fs::read_dir(&workflows_dir)? {
                    let path = entry?.path();
                    if let Some(ext) = path.extension()
                        && (ext == "yml" || ext == "yaml")
                    {
                        workflows.push(path);
                    }
                }
            }
        }

        workflows.sort();
        Ok(workflows)
    }

    fn read_file_content(path: &std::path::Path) -> Result<String, Box<dyn Error>> {
        fs::read_to_string(path).map_err(|e| e.into())
    }

    fn extract_name(content: &str) -> Option<String> {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("name:") {
                let name = rest.trim().trim_matches(|c| c == '"' || c == '\'');
                if !name.is_empty() {
                    return Some(name.to_string());
                }
            }
        }
        None
    }
}

impl WorkflowSourcePort for FilesystemWorkflowSource {
    fn read_workflow(
        &self,
        repository: &Repository,
        workflow_name: Option<&str>,
    ) -> Result<String, Box<dyn Error>> {
        let files = self.find_workflow_files(repository)?;

        if let Some(name) = workflow_name {
            // First try matching by workflow name (the `name:` field in YAML)
            for file in &files {
                let content = Self::read_file_content(file)?;
                if let Some(wf_name) = Self::extract_name(&content)
                    && wf_name == name
                {
                    return Ok(content);
                }
            }
            return Err(format!("workflow {:?} not found", name).into());
        }

        if let Some(file) = files.first() {
            Self::read_file_content(file)
        } else {
            Err("no workflow files found".into())
        }
    }

    fn read_all_workflows(&self, repository: &Repository) -> Result<Vec<String>, Box<dyn Error>> {
        let files = self.find_workflow_files(repository)?;
        let mut contents = Vec::new();

        for file in files {
            let content = Self::read_file_content(&file)?;
            contents.push(content);
        }

        Ok(contents)
    }

    fn list_actions(&self, repository: &Repository) -> Result<Vec<String>, Box<dyn Error>> {
        let files = self.find_workflow_files(repository)?;
        let mut actions = BTreeSet::new();

        for file in files {
            let content = Self::read_file_content(&file)?;
            for line in content.lines() {
                let trimmed = line.trim();
                let rest = if let Some(r) = trimmed.strip_prefix("- uses:") {
                    r
                } else if let Some(r) = trimmed.strip_prefix("uses:") {
                    r
                } else {
                    continue;
                };
                let action = rest.trim();
                if !action.is_empty() && !action.starts_with('#') {
                    actions.insert(action.to_string());
                }
            }
        }

        Ok(actions.into_iter().collect())
    }

    fn list_workflows(
        &self,
        repository: &Repository,
    ) -> Result<Vec<crate::application::dtos::WorkflowListItem>, Box<dyn Error>> {
        let files = self.find_workflow_files(repository)?;
        let mut items = Vec::new();

        for file in files {
            let content = Self::read_file_content(&file)?;
            if let Some(name) = Self::extract_name(&content) {
                items.push(crate::application::dtos::WorkflowListItem::new(
                    Some(name),
                    Some(file.to_string_lossy().to_string()),
                ));
            }
        }

        Ok(items)
    }
}

/// Convenience constructor matching the old FilesystemWorkflowFileParser pattern.
impl Default for FilesystemWorkflowSource {
    fn default() -> Self {
        use crate::infrastructure::workflows::workflow_directories::WORKFLOW_DIRECTORIES;
        Self::new(&WORKFLOW_DIRECTORIES)
    }
}

unsafe impl Send for FilesystemWorkflowSource {}
unsafe impl Sync for FilesystemWorkflowSource {}
