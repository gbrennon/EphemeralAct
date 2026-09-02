use ephemeral_act::{
    core::ports::outbound::workflow_file_parser::WorkflowFileParserPort,
    infrastructure::workflows::FilesystemWorkflowFileParser,
};

#[cfg(test)]
mod tests {
    use super::*;

    fn parser() -> FilesystemWorkflowFileParser {
        FilesystemWorkflowFileParser
    }

    #[test]
    fn extract_actions_collects_uses_refs_from_both_line_forms() {
        let yaml = "\
jobs:
  build:
    steps:
      - uses: actions/checkout@v4
      - uses: docker://node:20
  release:
    steps:
      - uses: ./local-action
";

        let actions = parser().extract_actions(yaml);

        assert_eq!(
            actions,
            vec!["./local-action", "actions/checkout@v4", "docker://node:20"]
        );
    }

    #[test]
    fn extract_actions_deduplicates_repeated_refs() {
        let yaml = "\
- uses: actions/checkout@v4
- uses: actions/checkout@v4
";

        let actions = parser().extract_actions(yaml);

        assert_eq!(actions, vec!["actions/checkout@v4"]);
    }

    #[test]
    fn extract_actions_skips_empty_and_comment_only_values() {
        let yaml = "\
- uses:
- uses: # pinned elsewhere
- uses: actions/checkout@v4
";

        let actions = parser().extract_actions(yaml);

        assert_eq!(actions, vec!["actions/checkout@v4"]);
    }

    #[test]
    fn extract_actions_returns_empty_for_yaml_without_uses() {
        let yaml = "name: ci\non: push\njobs:\n  build:\n    runs-on: ubuntu-latest\n";

        let actions = parser().extract_actions(yaml);

        assert!(actions.is_empty());
    }

    #[test]
    fn extract_summary_returns_unquoted_name() {
        let (name, _file) = parser().extract_summary("name: ci\njobs: {}");

        assert_eq!(name.as_deref(), Some("ci"));
    }

    #[test]
    fn extract_summary_strips_quotes_from_name() {
        let (name, _file) = parser().extract_summary("name: \"ci pipeline\"\njobs: {}");

        assert_eq!(name.as_deref(), Some("ci pipeline"));
    }

    #[test]
    fn extract_summary_returns_none_when_name_absent() {
        let (name, _file) = parser().extract_summary("on: push\njobs: {}");

        assert_eq!(name, None);
    }
}
