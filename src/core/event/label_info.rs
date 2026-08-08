use serde::Serialize;

/// Label information.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LabelInfo {
    pub name: String,
    pub color: String,
}
