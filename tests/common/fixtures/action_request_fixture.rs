#![allow(dead_code)]
use std::{collections::HashMap, path::PathBuf, sync::Arc};

use ephact::core::{
    dtos::ExecuteActionRequest, expression::EvalContext, ports::outbound::ContainerPort,
};

use crate::common::fakes::stub_container::StubContainer;

/// Builds action execution requests for tests that only care about which
/// action was requested.
pub struct ActionRequestFixture;

impl ActionRequestFixture {
    pub fn for_action(action_ref: &str) -> ExecuteActionRequest {
        let container: Arc<dyn ContainerPort> = Arc::new(StubContainer);
        ExecuteActionRequest {
            action_ref: action_ref.to_string(),
            step: serde_yaml::from_str(&format!("uses: {action_ref}\n")).unwrap(),
            repo_path: PathBuf::from("/workspace"),
            env: HashMap::new(),
            context: EvalContext::new(),
            container,
        }
    }
}
