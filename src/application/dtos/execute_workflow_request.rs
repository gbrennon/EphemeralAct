use std::path::Path;

use crate::domain::expression::EvalContext;

/// Infrastructure-facing request carrying already-resolved workflow content.
pub struct ExecuteWorkflowRequest<'a> {
    pub workflow_content: &'a str,
    pub repo_path: &'a Path,
    pub context: &'a EvalContext,
}
