use std::collections::HashMap;

use serde::Deserialize;

use super::{EventConfig, OnVisitor};

/// The event(s) that trigger a workflow.
///
/// GitHub Actions supports three forms for the `on` field:
/// - **Scalar**: `on: push`
/// - **Sequence**: `on: [push, pull_request]`
/// - **Mapping**: `on: { push: { branches: [main] } }`
///
/// This enum handles all three via serde's untagged deserialization.
///
/// # Examples
///
/// ```
/// use ephact::domain::workflow::On;
///
/// // Scalar form
/// let on: On = serde_yaml::from_str("push").unwrap();
/// assert!(on.is_single("push"));
///
/// // Sequence form
/// let on: On = serde_yaml::from_str("[push, pull_request]").unwrap();
/// assert!(on.is_multiple());
///
/// // Mapping form
/// let on: On = serde_yaml::from_str("{push: {branches: [main]}}").unwrap();
/// assert!(on.has_event("push"));
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum On {
    /// A single event name (e.g. `on: push`).
    Single(String),
    /// Multiple event names (e.g. `on: [push, pull_request]`).
    Multiple(Vec<String>),
    /// Event names with type-specific configuration
    /// (e.g. `on: { push: { branches: [main] } }`).
    WithTypes(HashMap<String, Option<EventConfig>>),
}

impl On {
    /// Returns `true` if the `on` field matches a single event name.
    pub fn is_single(&self, name: &str) -> bool {
        matches!(self, On::Single(n) if n == name)
    }

    /// Returns `true` if the `on` field contains multiple events.
    pub fn is_multiple(&self) -> bool {
        matches!(self, On::Multiple(_))
    }

    /// Returns `true` if the `on` field has type-specific configuration.
    pub fn has_types(&self) -> bool {
        matches!(self, On::WithTypes(_))
    }

    /// Returns `true` if the given event name is present in any form.
    pub fn has_event(&self, name: &str) -> bool {
        match self {
            On::Single(n) => n == name,
            On::Multiple(names) => names.iter().any(|n| n == name),
            On::WithTypes(map) => map.contains_key(name),
        }
    }

    /// Returns all event names regardless of form.
    pub fn event_names(&self) -> Vec<&str> {
        match self {
            On::Single(name) => vec![name.as_str()],
            On::Multiple(names) => names.iter().map(|s| s.as_str()).collect(),
            On::WithTypes(map) => map.keys().map(|s| s.as_str()).collect(),
        }
    }
}

impl Default for On {
    fn default() -> Self {
        On::Single("push".to_owned())
    }
}

impl<'de> Deserialize<'de> for On {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(OnVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_scalar_on() {
        let on: On = serde_yaml::from_str("push").unwrap();
        assert!(on.is_single("push"));
        assert!(!on.is_single("pull_request"));
        assert_eq!(on.event_names(), vec!["push"]);
    }

    #[test]
    fn deserialize_sequence_on() {
        let on: On = serde_yaml::from_str("[push, pull_request]").unwrap();
        assert!(on.is_multiple());
        assert!(on.has_event("push"));
        assert!(on.has_event("pull_request"));
        assert_eq!(on.event_names(), vec!["push", "pull_request"]);
    }

    #[test]
    fn deserialize_mapping_on() {
        let yaml = r#"
push:
  branches: [main, develop]
pull_request:
  types: [opened, synchronize]
"#;
        let on: On = serde_yaml::from_str(yaml).unwrap();
        assert!(on.has_types());
        assert!(on.has_event("push"));
        assert!(on.has_event("pull_request"));
        assert!(!on.has_event("schedule"));
    }

    #[test]
    fn deserialize_mapping_with_null_config() {
        let yaml = "push:\npull_request:\n";
        let on: On = serde_yaml::from_str(yaml).unwrap();
        assert!(on.has_event("push"));
        assert!(on.has_event("pull_request"));
    }

    #[test]
    fn deserialize_workflow_dispatch_with_inputs() {
        let yaml = r#"
workflow_dispatch:
  inputs:
    name:
      description: 'Name to greet'
      required: true
      type: string
    environment:
      description: 'Target environment'
      required: false
      default: 'staging'
      type: choice
      options: [staging, production]
"#;
        let on: On = serde_yaml::from_str(yaml).unwrap();
        assert!(on.has_event("workflow_dispatch"));
    }

    #[test]
    fn deserialize_schedule_with_cron() {
        let yaml = r#"
schedule:
  cron: ['0 0 * * *']
"#;
        let on: On = serde_yaml::from_str(yaml).unwrap();
        assert!(on.has_event("schedule"));
    }

    #[test]
    fn has_event_for_single() {
        let on: On = serde_yaml::from_str("push").unwrap();
        assert!(on.has_event("push"));
        assert!(!on.has_event("pull_request"));
    }

    #[test]
    fn event_names_for_with_types() {
        let yaml = "push:\npull_request:\n";
        let on: On = serde_yaml::from_str(yaml).unwrap();
        let mut names = on.event_names();
        names.sort();
        assert_eq!(names, vec!["pull_request", "push"]);
    }

    #[test]
    fn default_is_single_push() {
        assert_eq!(On::default(), On::Single("push".to_owned()));
    }

    #[test]
    fn deserialize_invalid_type_errors() {
        let result: Result<On, _> = serde_yaml::from_str("123");
        assert!(result.is_err());
    }

    #[test]
    fn deserialize_from_value_string() {
        let value: serde_yaml::Value = serde_yaml::from_str("push").unwrap();
        let on: On = serde_yaml::from_value(value).unwrap();
        assert_eq!(on, On::Single("push".to_owned()));
    }
}
