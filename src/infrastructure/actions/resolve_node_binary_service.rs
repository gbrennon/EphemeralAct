use crate::infrastructure::actions::resolve_node_binary_port::ResolveNodeBinaryPort;
use std::collections::HashMap;

use crate::application::dtos::ResolveNodeBinaryRequest;

/// Interpreter used for JavaScript actions when the container exposes no
/// absolute path for it.
const NODE_COMMAND: &str = "node";

/// Service that finds the node interpreter to run a JavaScript action with.
///
/// Runner images commonly install node in a tool cache that only a login shell
/// puts on `PATH`, so the binary is looked up through one; when the lookup
/// finds nothing, the bare command is used so the failure names the missing
/// interpreter.
pub struct ResolveNodeBinaryService;

impl ResolveNodeBinaryService {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ResolveNodeBinaryService {
    fn default() -> Self {
        Self::new()
    }
}

impl ResolveNodeBinaryPort for ResolveNodeBinaryService {
    fn execute(&self, request: ResolveNodeBinaryRequest<'_>) -> String {
        request
            .container
            .exec(
                &["bash".into(), "-lc".into(), "command -v node".into()],
                None,
                &HashMap::new(),
            )
            .ok()
            .filter(|result| result.exit_code == 0)
            .map(|result| result.stdout.trim().to_string())
            .filter(|path| !path.is_empty())
            .unwrap_or_else(|| NODE_COMMAND.to_string())
    }
}
