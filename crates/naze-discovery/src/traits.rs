use axum::http::HeaderMap;
use std::collections::HashMap;

use crate::types::*;

// ─── Storage Error ──────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct StorageError {
    pub message: String,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StorageError: {}", self.message)
    }
}

impl std::error::Error for StorageError {}

impl StorageError {
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

// ─── TrustScorer ────────────────────────────────────────────────────────────

/// Score trust for a service against a profile.
pub trait TrustScorer: Send + Sync {
    fn score(&self, input: &TrustInput) -> TrustOutput;
    fn name(&self) -> &str;
}

// ─── CapabilityMatcher ──────────────────────────────────────────────────────

/// Match services against a structural capability query.
pub trait CapabilityMatcherTrait: Send + Sync {
    fn search(
        &self,
        query: &CapabilityQuery,
        storage: &dyn StorageBackend,
    ) -> Vec<MatchResult>;
    fn name(&self) -> &str;
}

// ─── CapabilityExtractor ────────────────────────────────────────────────────

/// Extract capabilities from a manifest.
pub trait CapabilityExtractor: Send + Sync {
    fn extract(&self, manifest: &serde_json::Value) -> Vec<Capability>;
    fn name(&self) -> &str;
}

// ─── IdentityVerifier ───────────────────────────────────────────────────────

/// Verify publisher identity from request context.
pub trait IdentityVerifier: Send + Sync {
    fn verify(&self, headers: &HeaderMap) -> Result<Option<PublisherIdentity>, AuthError>;
    fn name(&self) -> &str;
}

// ─── FederationSync ─────────────────────────────────────────────────────────

/// Synchronize with a peer discovery node.
pub trait FederationSync: Send + Sync {
    fn sync(
        &self,
        peer_url: &str,
        storage: &dyn StorageBackend,
    ) -> Result<SyncResult, SyncError>;
    fn name(&self) -> &str;
}

// ─── StorageBackend ─────────────────────────────────────────────────────────

/// Persist and retrieve all discovery data.
/// All methods use ServiceRef (domain, name) — never local integer IDs.
pub trait StorageBackend: Send + Sync {
    // Services
    fn upsert_service(&self, service: &ServiceRecord) -> Result<ServiceRef, StorageError>;
    fn get_service(&self, service: &ServiceRef) -> Result<Option<ServiceRecord>, StorageError>;
    fn deactivate_service(&self, service: &ServiceRef) -> Result<(), StorageError>;
    fn list_services(&self, filter: &ServiceFilter) -> Result<Vec<ServiceRecord>, StorageError>;

    // Capabilities
    fn replace_capabilities(
        &self,
        service: &ServiceRef,
        caps: &[Capability],
    ) -> Result<(), StorageError>;
    fn query_capabilities(
        &self,
        matchers: &[CapabilityMatcher],
    ) -> Result<Vec<ServiceRef>, StorageError>;

    // Trust
    fn upsert_trust_score(
        &self,
        service: &ServiceRef,
        profile: &str,
        output: &TrustOutput,
    ) -> Result<(), StorageError>;
    fn get_trust_scores(
        &self,
        service: &ServiceRef,
    ) -> Result<HashMap<String, TrustOutput>, StorageError>;

    // Observations
    fn record_observation(&self, obs: &Observation) -> Result<(), StorageError>;
    fn get_observation_signals(
        &self,
        service: &ServiceRef,
    ) -> Result<ObservationSignals, StorageError>;

    // Compositions
    fn upsert_composition(&self, services: &[ServiceRef]) -> Result<(), StorageError>;
    fn get_top_patterns(&self, limit: u32) -> Result<Vec<CompositionPattern>, StorageError>;

    // Provenance
    fn set_provenance(
        &self,
        service: &ServiceRef,
        sources: &[ServiceRef],
    ) -> Result<(), StorageError>;
    fn get_provenance(&self, service: &ServiceRef) -> Result<Vec<ServiceRef>, StorageError>;

    // Versions
    fn archive_version(&self, service: &ServiceRef) -> Result<(), StorageError>;
    fn list_versions(&self, service: &ServiceRef) -> Result<Vec<VersionRecord>, StorageError>;

    // Peers
    fn add_peer(&self, peer: &PeerRecord) -> Result<String, StorageError>;
    fn list_peers(&self) -> Result<Vec<PeerRecord>, StorageError>;
    fn remove_peer(&self, peer_url: &str) -> Result<(), StorageError>;

    // Profiles
    fn list_profiles(&self) -> Result<Vec<TrustProfile>, StorageError>;
    fn create_profile(&self, profile: &TrustProfile) -> Result<(), StorageError>;

    // Export
    fn export_public_services(
        &self,
        since: Option<&str>,
    ) -> Result<Vec<ServiceExport>, StorageError>;

    // Info
    fn get_stats(&self) -> Result<(u64, u64), StorageError>; // (service_count, peer_count)

    fn name(&self) -> &str;
}
