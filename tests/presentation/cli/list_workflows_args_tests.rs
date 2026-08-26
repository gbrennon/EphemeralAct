use std::path::PathBuf;

use ephemeral_act::core::dtos::ListWorkflowsRequest;
use ephemeral_act::presentation::cli::parse_list_workflows_test_args;

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
