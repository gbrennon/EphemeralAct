#![allow(dead_code)]
use ephact::{
    application::dtos::ResolveNodeBinaryRequest,
    infrastructure::actions::resolve_node_binary_port::ResolveNodeBinaryPort,
};

/// Reports a prepared node interpreter.
#[derive(Clone)]
pub struct FakeResolveNodeBinaryPort {
    binary: String,
}

impl FakeResolveNodeBinaryPort {
    pub fn returning(binary: &str) -> Self {
        Self {
            binary: binary.to_string(),
        }
    }
}

impl ResolveNodeBinaryPort for FakeResolveNodeBinaryPort {
    fn execute(&self, _request: ResolveNodeBinaryRequest<'_>) -> String {
        self.binary.clone()
    }
}
