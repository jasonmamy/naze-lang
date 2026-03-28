use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── Global Identity ────────────────────────────────────────────────────────

/// Global service identity — (domain, name) is unique across all nodes.
/// The reference impl maps this to a local i64 internally, but the trait
/// boundary never exposes local IDs.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct ServiceRef {
    pub domain: String,
    pub name: String,
}

// ─── Trust Types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustProfile {
    pub name: String,
    pub weights: TrustWeights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustWeights {
    pub external_domains: f64,
    pub personal_data: f64,
    pub device_apis: f64,
    pub data_flow: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustInput {
    pub manifest: serde_json::Value,
    pub profile: TrustProfile,
    pub signals: ObservationSignals,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustOutput {
    pub score: f64,
    pub base_score: f64,
    pub adjustment: f64,
    pub breakdown: HashMap<String, f64>,
    pub scorer: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObservationSignals {
    pub usage_count: u64,
    pub discovery_count: u64,
    pub flag_count: u64,
    pub flag_reasons: Vec<String>,
    pub composition_count: u64,
    pub last_activity: Option<String>,
    pub days_since_activity: u64,
    pub source_flag_count: u64,
}

// ─── Capability Types ───────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub kind: String,
    pub name: String,
    pub value_type: Option<String>,
    pub metadata: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityMatcher {
    pub kind: String,
    pub name: Option<String>,
    pub name_like: Option<String>,
    pub value_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityQuery {
    pub require: Vec<CapabilityMatcher>,
    #[serde(default)]
    pub prefer: Option<Vec<CapabilityMatcher>>,
    pub trust_profile: Option<String>,
    pub min_trust: Option<f64>,
    pub limit: Option<u32>,
}

// ─── Service Types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRecord {
    pub domain: String,
    pub name: String,
    pub version: String,
    pub manifest_hash: String,
    pub manifest: serde_json::Value,
    pub headless_hash: Option<String>,
    pub headless: Option<Vec<u8>>,
    pub visibility: String,
    pub publisher: Option<String>,
    pub active: bool,
    pub registered_at: Option<String>,
    pub updated_at: Option<String>,
    pub last_activity: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceFilter {
    pub active_only: bool,
    pub visibility: Option<String>,
    pub domain: Option<String>,
}

// ─── API Request/Response Types ─────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub domain: String,
    pub manifest: serde_json::Value,
    pub headless: Option<String>, // base64-encoded
    pub visibility: Option<String>,
    pub publisher: Option<String>,
    pub composed_from: Option<Vec<ServiceRef>>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub domain: String,
    pub name: String,
    pub version: String,
    pub manifest_hash: String,
    pub trust_scores: HashMap<String, f64>,
    pub capabilities_indexed: usize,
    pub registered_at: String,
}

#[derive(Debug, Serialize)]
pub struct SearchResult {
    pub domain: String,
    pub name: String,
    pub version: String,
    pub trust_score: f64,
    pub manifest_hash: String,
    pub headless_hash: Option<String>,
    pub matched_capabilities: Vec<MatchedCapability>,
    pub preferred_matches: u32,
    pub publisher: Option<String>,
    pub manifest_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchedCapability {
    pub kind: String,
    pub name: String,
    pub value_type: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: u32,
}

#[derive(Debug, Deserialize)]
pub struct ObserveRequest {
    pub kind: String,
    pub service_domain: String,
    pub service_name: String,
    pub agent_id: Option<String>,
    pub observation_id: Option<String>,
    pub payload: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct FlagRequest {
    pub service_domain: String,
    pub service_name: String,
    pub reason: String,
    pub evidence: Option<String>,
    pub agent_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ComposeRequest {
    pub services: Vec<ServiceRef>,
    pub agent_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SignalsSummary {
    pub discovery_count: u64,
    pub usage_count: u64,
    pub flag_count: u64,
    pub composition_count: u64,
    pub last_activity: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CompositionPattern {
    pub services: Vec<ServiceRef>,
    pub frequency: u64,
    pub first_seen: String,
    pub last_seen: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerRecord {
    pub url: String,
    pub name: Option<String>,
    pub trust_profile: Option<String>,
    pub last_sync: Option<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRecord {
    pub version: String,
    pub manifest_hash: String,
    pub headless_hash: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub version: String,
    pub network_id: String,
    pub scope: String,
    pub services: u64,
    pub peers: u64,
    pub profiles: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ServiceExport {
    pub domain: String,
    pub name: String,
    pub version: String,
    pub manifest: serde_json::Value,
    pub manifest_hash: String,
    pub headless_hash: Option<String>,
    pub publisher: Option<String>,
    pub updated_at: String,
}

// ─── Identity Types ─────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct PublisherIdentity {
    pub id: String,
}

#[derive(Debug)]
pub struct AuthError {
    pub message: String,
}

// ─── Sync Types ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct SyncResult {
    pub added: u64,
    pub updated: u64,
    pub conflicts: u64,
}

#[derive(Debug)]
pub struct SyncError {
    pub message: String,
}

// ─── Observation (internal) ─────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct Observation {
    pub observation_id: Option<String>,
    pub kind: String,
    pub service: ServiceRef,
    pub agent_id: Option<String>,
    pub payload: serde_json::Value,
}

// ─── Match Result (internal) ────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub service: ServiceRef,
    pub matched_capabilities: Vec<MatchedCapability>,
    pub preferred_matches: u32,
}
