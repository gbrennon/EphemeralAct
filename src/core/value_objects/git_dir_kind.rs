#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitDirKind {
    Standalone,
    Worktree,
}
