use ephemeral_act::presentation::cli::{RunArgs, parse_run_test_args};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_key_value_with_equals() {
        let (k, v) = RunArgs::parse_key_value("KEY=value").unwrap();
        assert_eq!(k, "KEY");
        assert_eq!(v, "value");
    }

    #[test]
    fn parse_key_value_missing_equals() {
        let err = RunArgs::parse_key_value("no_equals").unwrap_err();
        assert!(err.contains("KEY=VALUE"));
    }

    #[test]
    fn to_domain_defaults() {
        let args = parse_run_test_args(&[]);
        let (_config, repo) = args.to_domain().unwrap();
        assert!(!repo.name().as_str().is_empty());
    }

    #[test]
    fn to_domain_with_workflow() {
        let args = parse_run_test_args(&["--workflow", "ci.yml"]);
        let (config, _repo) = args.to_domain().unwrap();
        assert_eq!(config.workflow().unwrap().as_str(), "ci.yml");
    }

    #[test]
    fn to_domain_with_job() {
        let args = parse_run_test_args(&["--job", "test"]);
        let (config, _repo) = args.to_domain().unwrap();
        assert_eq!(config.job().unwrap().as_str(), "test");
    }

    #[test]
    fn to_domain_with_event() {
        let args = parse_run_test_args(&["--event", "push"]);
        let (config, _repo) = args.to_domain().unwrap();
        assert_eq!(config.event().unwrap().as_str(), "push");
    }

    #[test]
    fn to_domain_with_inputs() {
        let args = parse_run_test_args(&["--input", "VAR1=val1", "--input", "VAR2=val2"]);
        let (config, _repo) = args.to_domain().unwrap();
        assert_eq!(config.inputs().len(), 2);
    }

    #[test]
    fn to_domain_with_secrets() {
        let args = parse_run_test_args(&["--secret", "TOKEN=abc123", "--secret", "PASS=xyz"]);
        let (config, _repo) = args.to_domain().unwrap();
        assert_eq!(config.secrets().len(), 2);
    }

    #[test]
    fn to_domain_defaults_allow_flags_to_disabled() {
        let args = parse_run_test_args(&[]);
        let (config, _repo) = args.to_domain().unwrap();
        assert!(!config.allow_real_container());
        assert!(!config.allow_real_fetcher());
        assert!(!config.allow_network());
    }

    #[test]
    fn to_domain_with_allow_real_container() {
        let args = parse_run_test_args(&["--allow-real-container"]);
        let (config, _repo) = args.to_domain().unwrap();
        assert!(config.allow_real_container());
    }

    #[test]
    fn to_domain_with_allow_real_fetcher() {
        let args = parse_run_test_args(&["--allow-real-fetcher"]);
        let (config, _repo) = args.to_domain().unwrap();
        assert!(config.allow_real_fetcher());
    }

    #[test]
    fn to_domain_with_allow_network() {
        let args = parse_run_test_args(&["--allow-network"]);
        let (config, _repo) = args.to_domain().unwrap();
        assert!(config.allow_network());
    }

    #[test]
    fn to_domain_with_allow_flags_alongside_workflow_and_event() {
        let args = parse_run_test_args(&[
            "--allow-real-container",
            "--allow-real-fetcher",
            "--workflow",
            "ci.yml",
            "--event",
            "pull_request",
        ]);
        let (config, _repo) = args.to_domain().unwrap();
        assert!(config.allow_real_container());
        assert!(config.allow_real_fetcher());
        assert_eq!(config.workflow().unwrap().as_str(), "ci.yml");
        assert_eq!(config.event().unwrap().as_str(), "pull_request");
    }

    #[test]
    fn to_domain_keeps_secret_name_and_value() {
        let args = parse_run_test_args(&["--secret", "TOKEN=abc123"]);
        let (config, _repo) = args.to_domain().unwrap();
        assert_eq!(config.secrets()[0].name(), "TOKEN");
        assert_eq!(config.secrets()[0].value(), "abc123");
    }

    #[test]
    fn parse_secret_reads_value_from_environment_when_omitted() {
        unsafe { std::env::set_var("EPHEMERAL_ACT_TEST_SECRET", "from-env") };
        let (name, value) = RunArgs::parse_secret("EPHEMERAL_ACT_TEST_SECRET").unwrap();
        assert_eq!(name, "EPHEMERAL_ACT_TEST_SECRET");
        assert_eq!(value, "from-env");
    }

    #[test]
    fn parse_secret_errors_when_no_value_is_available() {
        let err = RunArgs::parse_secret("EPHEMERAL_ACT_ABSENT_SECRET").unwrap_err();
        assert!(err.contains("no value"), "{err}");
    }

    #[test]
    fn to_domain_with_all_workflows() {
        let args = parse_run_test_args(&["--all-workflows"]);
        let (config, _repo) = args.to_domain().unwrap();
        assert!(config.all_workflows());
    }

    #[test]
    fn to_domain_without_all_workflows_defaults_to_disabled() {
        let args = parse_run_test_args(&[]);
        let (config, _repo) = args.to_domain().unwrap();
        assert!(!config.all_workflows());
    }
}
