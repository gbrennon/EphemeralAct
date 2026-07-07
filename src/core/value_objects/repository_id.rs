use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RepositoryId(pub Uuid);

impl RepositoryId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_non_nil_uuid() {
        let id = RepositoryId::new();
        assert!(!id.as_uuid().is_nil());
    }

    #[test]
    fn new_creates_unique_ids() {
        let id1 = RepositoryId::new();
        let id2 = RepositoryId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn as_uuid_returns_inner_uuid() {
        let id = RepositoryId::new();
        let uuid = id.as_uuid();
        assert_eq!(uuid, &id.0);
    }
}