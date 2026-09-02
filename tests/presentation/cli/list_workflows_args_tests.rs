use std::path::PathBuf;

use ephact::{core::dtos::ListWorkflowsRequest, presentation::cli::parse_list_workflows_test_args};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_domain_returns_ok() {
        let args = parse_list_workflows_test_args(&[]);

        let result = args.to_domain();
        assert_eq!(
            result.unwrap(),
            ListWorkflowsRequest::new(PathBuf::from("."))
        );
    }
}
