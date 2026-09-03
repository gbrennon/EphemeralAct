/// One workflow file discovered in a repository, as the presentation layer
/// displays it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkflowListItem {
    /// Display name declared by the workflow, when it declares one.
    pub name: Option<String>,
    /// File name the workflow was read from, when it is known.
    pub file: Option<String>,
}

impl WorkflowListItem {
    /// Creates an item from the name and file name of a workflow.
    pub fn new(name: Option<String>, file: Option<String>) -> Self {
        Self { name, file }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_workflow_list_item_keeps_fields() {
        let item = WorkflowListItem::new(Some("ci".into()), Some("ci.yml".into()));
        assert_eq!(item.name.as_deref(), Some("ci"));
        assert_eq!(item.file.as_deref(), Some("ci.yml"));
    }

    #[test]
    fn new_workflow_list_item_allows_missing_fields() {
        let item = WorkflowListItem::new(None, None);
        assert_eq!(item.name, None);
        assert_eq!(item.file, None);
    }
}
