use ephemeral_act::presentation::cli::{parse_run_test_args, RunArgs};

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
fn to_domain_with_extra_args() {
    let args = parse_run_test_args(&["--extra-arg", "verbose", "--extra-arg", "dryrun"]);
    let (config, _repo) = args.to_domain().unwrap();
    assert_eq!(config.extra_args().len(), 2);
}
