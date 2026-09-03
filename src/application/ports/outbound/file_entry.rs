/// A file entry for copy operations.
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Path inside the container
    pub path: String,
    /// File content as raw bytes
    pub content: Vec<u8>,
    /// Unix file mode (e.g. 0o644)
    pub mode: u32,
}
