use crate::traits::{FederationSync, StorageBackend};
use crate::types::{SyncError, SyncResult};

pub struct StubSync;

impl StubSync {
    pub fn new() -> Self {
        Self
    }
}

impl FederationSync for StubSync {
    fn sync(
        &self,
        _peer_url: &str,
        _storage: &dyn StorageBackend,
    ) -> Result<SyncResult, SyncError> {
        Err(SyncError {
            message: "federation sync not implemented in reference server".into(),
        })
    }

    fn name(&self) -> &str {
        "stub-v1"
    }
}
