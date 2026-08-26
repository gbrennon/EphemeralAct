use std::path::PathBuf;

use ephemeral_act::{
    core::dtos::ListActionsRequest, presentation::cli::parse_list_actions_test_args,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_domain_returns_ok() {
        let args = parse_list_actions_test_args(&[]);

        let result = args.to_domain();
        assert_eq!(result.unwrap(), ListActionsRequest::new(PathBuf::from(".")));
    }
}
