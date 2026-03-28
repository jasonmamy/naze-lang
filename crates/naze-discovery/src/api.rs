use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::dashboard;
use crate::traits::*;
use crate::types::*;

pub struct AppState {
    pub storage: Box<dyn StorageBackend>,
    pub scorer: Box<dyn TrustScorer>,
    pub matcher: Box<dyn CapabilityMatcherTrait>,
    pub extractor: Box<dyn CapabilityExtractor>,
    pub identity: Box<dyn IdentityVerifier>,
    pub sync: Box<dyn FederationSync>,
    pub network_id: String,
    pub scope: String,
}

type S = Arc<AppState>;

fn err(status: StatusCode, msg: &str) -> axum::http::Response<Body> {
    (status, Json(serde_json::json!({"error": msg}))).into_response()
}

use axum::body::Body;

pub fn router(state: AppState) -> Router {
    let state = Arc::new(state);

    Router::new()
        // Health & Info
        .route("/health", get(health))
        .route("/api/v1/discovery/info", get(info))
        // Service Registration
        .route("/api/v1/discovery/services", post(register_service))
        .route(
            "/api/v1/discovery/services/:domain/:name",
            get(get_service),
        )
        .route(
            "/api/v1/discovery/services/:domain/:name",
            delete(deactivate_service),
        )
        .route(
            "/api/v1/discovery/services/:domain/:name/manifest",
            get(get_manifest),
        )
        .route(
            "/api/v1/discovery/services/:domain/:name/trust",
            get(get_trust_scores),
        )
        .route(
            "/api/v1/discovery/services/:domain/:name/trust/:profile",
            get(get_trust_score),
        )
        .route(
            "/api/v1/discovery/services/:domain/:name/signals",
            get(get_signals),
        )
        .route(
            "/api/v1/discovery/services/:domain/:name/provenance",
            get(get_provenance),
        )
        .route(
            "/api/v1/discovery/services/:domain/:name/versions",
            get(get_versions),
        )
        // Capability Discovery
        .route("/api/v1/discovery/search", post(search_capabilities))
        .route("/api/v1/discovery/search", get(search_text))
        // Trust Profiles
        .route("/api/v1/discovery/profiles", get(list_profiles))
        .route("/api/v1/discovery/profiles", post(create_profile))
        // Observations
        .route("/api/v1/discovery/observe", post(record_observation))
        .route("/api/v1/discovery/flag", post(flag_service))
        .route("/api/v1/discovery/compose", post(record_composition))
        // Emergence
        .route("/api/v1/discovery/patterns", get(get_patterns))
        .route("/api/v1/discovery/trending", get(get_trending))
        // Federation
        .route("/api/v1/discovery/peers", get(list_peers))
        .route("/api/v1/discovery/peers", post(add_peer))
        .route("/api/v1/discovery/peers/:id", delete(remove_peer))
        .route("/api/v1/discovery/peers/sync", post(sync_peer))
        .route("/api/v1/discovery/export", get(export_services))
        // Admin
        .route("/api/v1/discovery/recompute", post(recompute_trust))
        // Dashboard
        .merge(dashboard::routes())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

// ─── Health & Info ──────────────────────────────────────────────────────────

async fn health() -> impl IntoResponse {
    (StatusCode::OK, "ok")
}

async fn info(State(state): State<S>) -> impl IntoResponse {
    let stats = state.storage.get_stats().unwrap_or((0, 0));
    let profiles: Vec<String> = state
        .storage
        .list_profiles()
        .unwrap_or_default()
        .into_iter()
        .map(|p| p.name)
        .collect();

    Json(ServerInfo {
        version: "0.1.0".into(),
        network_id: state.network_id.clone(),
        scope: state.scope.clone(),
        services: stats.0,
        peers: stats.1,
        profiles,
    })
}

// ─── Service Registration ───────────────────────────────────────────────────

async fn register_service(
    State(state): State<S>,
    Json(req): Json<RegisterRequest>,
) -> impl IntoResponse {
    // Extract name from manifest
    let name = req
        .manifest
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or("unnamed")
        .to_string();

    let version = req
        .manifest
        .get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("0.1.0")
        .to_string();

    // Compute manifest hash
    let manifest_str = serde_json::to_string(&req.manifest).unwrap_or_default();
    let manifest_hash = format!("{:x}", Sha256::digest(manifest_str.as_bytes()));

    // Decode headless binary if present
    let (headless, headless_hash) = if let Some(ref b64) = req.headless {
        match base64::Engine::decode(&base64::engine::general_purpose::STANDARD, b64) {
            Ok(data) => {
                let hash = format!("{:x}", Sha256::digest(&data));
                (Some(data), Some(hash))
            }
            Err(_) => return err(StatusCode::BAD_REQUEST, "invalid base64 in headless field"),
        }
    } else {
        (None, None)
    };

    let sref = ServiceRef {
        domain: req.domain.clone(),
        name: name.clone(),
    };

    // Archive old version if service exists
    if state.storage.get_service(&sref).ok().flatten().is_some() {
        let _ = state.storage.archive_version(&sref);
    }

    let record = ServiceRecord {
        domain: req.domain.clone(),
        name: name.clone(),
        version: version.clone(),
        manifest_hash: manifest_hash.clone(),
        manifest: req.manifest.clone(),
        headless_hash: headless_hash.clone(),
        headless,
        visibility: req.visibility.unwrap_or_else(|| "public".into()),
        publisher: req.publisher,
        active: true,
        registered_at: None,
        updated_at: None,
        last_activity: None,
    };

    if let Err(e) = state.storage.upsert_service(&record) {
        return err(StatusCode::INTERNAL_SERVER_ERROR, &e.message);
    }

    // Extract and store capabilities
    let caps = state.extractor.extract(&req.manifest);
    let cap_count = caps.len();
    let _ = state.storage.replace_capabilities(&sref, &caps);

    // Set provenance if provided
    if let Some(ref sources) = req.composed_from {
        let _ = state.storage.set_provenance(&sref, sources);
    }

    // Compute trust scores against all profiles
    let profiles = state.storage.list_profiles().unwrap_or_default();
    let signals = state
        .storage
        .get_observation_signals(&sref)
        .unwrap_or_default();
    let mut trust_scores = HashMap::new();

    for profile in &profiles {
        let input = TrustInput {
            manifest: req.manifest.clone(),
            profile: profile.clone(),
            signals: signals.clone(),
        };
        let output = state.scorer.score(&input);
        trust_scores.insert(profile.name.clone(), output.score);
        let _ = state
            .storage
            .upsert_trust_score(&sref, &profile.name, &output);
    }

    let now = chrono::Utc::now().to_rfc3339();

    (
        StatusCode::CREATED,
        Json(RegisterResponse {
            domain: req.domain,
            name,
            version,
            manifest_hash,
            trust_scores,
            capabilities_indexed: cap_count,
            registered_at: now,
        }),
    )
        .into_response()
}

async fn get_service(
    State(state): State<S>,
    Path((domain, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let sref = ServiceRef { domain, name };
    match state.storage.get_service(&sref) {
        Ok(Some(svc)) => {
            let scores = state.storage.get_trust_scores(&sref).unwrap_or_default();
            let trust: HashMap<String, f64> = scores.into_iter().map(|(k, v)| (k, v.score)).collect();
            Json(serde_json::json!({
                "domain": svc.domain,
                "name": svc.name,
                "version": svc.version,
                "manifest_hash": svc.manifest_hash,
                "headless_hash": svc.headless_hash,
                "visibility": svc.visibility,
                "publisher": svc.publisher,
                "active": svc.active,
                "trust_scores": trust,
                "registered_at": svc.registered_at,
                "updated_at": svc.updated_at,
                "last_activity": svc.last_activity,
            }))
            .into_response()
        }
        Ok(None) => err(StatusCode::NOT_FOUND, "service not found"),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

async fn deactivate_service(
    State(state): State<S>,
    Path((domain, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let sref = ServiceRef { domain, name };
    match state.storage.deactivate_service(&sref) {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({"status": "deactivated"}))).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

async fn get_manifest(
    State(state): State<S>,
    Path((domain, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let sref = ServiceRef { domain, name };
    match state.storage.get_service(&sref) {
        Ok(Some(svc)) => Json(svc.manifest).into_response(),
        Ok(None) => err(StatusCode::NOT_FOUND, "service not found"),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

// ─── Trust ──────────────────────────────────────────────────────────────────

async fn get_trust_scores(
    State(state): State<S>,
    Path((domain, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let sref = ServiceRef { domain, name };
    match state.storage.get_trust_scores(&sref) {
        Ok(scores) => Json(serde_json::json!({
            "domain": sref.domain,
            "name": sref.name,
            "scores": scores,
        }))
        .into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

async fn get_trust_score(
    State(state): State<S>,
    Path((domain, name, profile)): Path<(String, String, String)>,
) -> impl IntoResponse {
    let sref = ServiceRef { domain, name };
    match state.storage.get_trust_scores(&sref) {
        Ok(scores) => match scores.get(&profile) {
            Some(output) => Json(output.clone()).into_response(),
            None => err(StatusCode::NOT_FOUND, "trust profile not found for this service"),
        },
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

async fn list_profiles(State(state): State<S>) -> impl IntoResponse {
    match state.storage.list_profiles() {
        Ok(profiles) => Json(profiles).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

async fn create_profile(
    State(state): State<S>,
    Json(profile): Json<TrustProfile>,
) -> impl IntoResponse {
    match state.storage.create_profile(&profile) {
        Ok(()) => (StatusCode::CREATED, Json(serde_json::json!({"status": "created", "name": profile.name}))).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

// ─── Capability Discovery ───────────────────────────────────────────────────

async fn search_capabilities(
    State(state): State<S>,
    Json(query): Json<CapabilityQuery>,
) -> impl IntoResponse {
    let results = state.matcher.search(&query, state.storage.as_ref());

    let search_results: Vec<SearchResult> = results
        .into_iter()
        .map(|r| {
            let trust_score = state
                .storage
                .get_trust_scores(&r.service)
                .ok()
                .and_then(|s| {
                    let p = query.trust_profile.as_deref().unwrap_or("default");
                    s.get(p).map(|t| t.score)
                })
                .unwrap_or(0.5);

            let svc = state.storage.get_service(&r.service).ok().flatten();
            let manifest_hash = svc.as_ref().map(|s| s.manifest_hash.clone()).unwrap_or_default();
            let headless_hash = svc.as_ref().and_then(|s| s.headless_hash.clone());
            let version = svc.as_ref().map(|s| s.version.clone()).unwrap_or_default();
            let publisher = svc.as_ref().and_then(|s| s.publisher.clone());

            SearchResult {
                domain: r.service.domain.clone(),
                name: r.service.name.clone(),
                version,
                trust_score,
                manifest_hash,
                headless_hash,
                matched_capabilities: r.matched_capabilities,
                preferred_matches: r.preferred_matches,
                publisher,
                manifest_url: format!(
                    "/api/v1/discovery/services/{}/{}/manifest",
                    r.service.domain,
                    urlencoding(&r.service.name),
                ),
            }
        })
        .collect();

    let total = search_results.len() as u32;
    Json(SearchResponse {
        results: search_results,
        total,
    })
}

fn urlencoding(s: &str) -> String {
    s.replace(' ', "%20")
}

#[derive(serde::Deserialize)]
struct TextSearchQuery {
    q: String,
    #[serde(default = "default_limit")]
    limit: u32,
    profile: Option<String>,
}

fn default_limit() -> u32 {
    20
}

async fn search_text(
    State(state): State<S>,
    Query(params): Query<TextSearchQuery>,
) -> impl IntoResponse {
    // Text fallback: search by name LIKE pattern
    let matcher = CapabilityMatcher {
        kind: "state_field".into(),
        name: None,
        name_like: Some(format!("%{}%", params.q)),
        value_type: None,
    };
    let fn_matcher = CapabilityMatcher {
        kind: "server_function".into(),
        name: None,
        name_like: Some(format!("%{}%", params.q)),
        value_type: None,
    };

    // Search state fields OR server functions matching the text
    let mut all_refs = Vec::new();
    if let Ok(refs) = state.storage.query_capabilities(&[matcher]) {
        all_refs.extend(refs);
    }
    if let Ok(refs) = state.storage.query_capabilities(&[fn_matcher]) {
        for r in refs {
            if !all_refs.contains(&r) {
                all_refs.push(r);
            }
        }
    }

    // Also search service names
    let all_services = state
        .storage
        .list_services(&ServiceFilter {
            active_only: true,
            ..Default::default()
        })
        .unwrap_or_default();

    for svc in &all_services {
        let sref = ServiceRef {
            domain: svc.domain.clone(),
            name: svc.name.clone(),
        };
        if svc.name.to_lowercase().contains(&params.q.to_lowercase()) && !all_refs.contains(&sref)
        {
            all_refs.push(sref);
        }
    }

    all_refs.truncate(params.limit as usize);

    let profile = params.profile.as_deref().unwrap_or("default");
    let results: Vec<SearchResult> = all_refs
        .into_iter()
        .map(|sref| {
            let trust_score = state
                .storage
                .get_trust_scores(&sref)
                .ok()
                .and_then(|s| s.get(profile).map(|t| t.score))
                .unwrap_or(0.5);
            let svc = state.storage.get_service(&sref).ok().flatten();

            SearchResult {
                domain: sref.domain.clone(),
                name: sref.name.clone(),
                version: svc.as_ref().map(|s| s.version.clone()).unwrap_or_default(),
                trust_score,
                manifest_hash: svc.as_ref().map(|s| s.manifest_hash.clone()).unwrap_or_default(),
                headless_hash: svc.as_ref().and_then(|s| s.headless_hash.clone()),
                matched_capabilities: Vec::new(),
                preferred_matches: 0,
                publisher: svc.as_ref().and_then(|s| s.publisher.clone()),
                manifest_url: format!(
                    "/api/v1/discovery/services/{}/{}/manifest",
                    sref.domain,
                    urlencoding(&sref.name),
                ),
            }
        })
        .collect();

    let total = results.len() as u32;
    Json(SearchResponse { results, total })
}

// ─── Observations ───────────────────────────────────────────────────────────

async fn record_observation(
    State(state): State<S>,
    Json(req): Json<ObserveRequest>,
) -> impl IntoResponse {
    let obs = Observation {
        observation_id: req.observation_id,
        kind: req.kind,
        service: ServiceRef {
            domain: req.service_domain,
            name: req.service_name,
        },
        agent_id: req.agent_id,
        payload: req.payload,
    };

    match state.storage.record_observation(&obs) {
        Ok(()) => (StatusCode::CREATED, Json(serde_json::json!({"status": "recorded"}))).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

const FLAG_DEACTIVATION_THRESHOLD: u64 = 5;

async fn flag_service(
    State(state): State<S>,
    Json(req): Json<FlagRequest>,
) -> impl IntoResponse {
    let sref = ServiceRef {
        domain: req.service_domain,
        name: req.service_name,
    };

    let obs = Observation {
        observation_id: None,
        kind: "flag".into(),
        service: sref.clone(),
        agent_id: req.agent_id,
        payload: serde_json::json!({"reason": req.reason, "evidence": req.evidence}),
    };

    if let Err(e) = state.storage.record_observation(&obs) {
        return err(StatusCode::INTERNAL_SERVER_ERROR, &e.message);
    }

    // Check if flag threshold reached
    let signals = state
        .storage
        .get_observation_signals(&sref)
        .unwrap_or_default();

    let mut deactivated = false;
    if signals.flag_count >= FLAG_DEACTIVATION_THRESHOLD {
        let _ = state.storage.deactivate_service(&sref);
        deactivated = true;
    }

    // Recompute trust with updated signals
    recompute_service_trust(&state, &sref);

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "status": "flagged",
            "flag_count": signals.flag_count,
            "deactivated": deactivated,
        })),
    )
        .into_response()
}

async fn record_composition(
    State(state): State<S>,
    Json(req): Json<ComposeRequest>,
) -> impl IntoResponse {
    match state.storage.upsert_composition(&req.services) {
        Ok(()) => (StatusCode::CREATED, Json(serde_json::json!({"status": "recorded"}))).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

async fn get_signals(
    State(state): State<S>,
    Path((domain, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let sref = ServiceRef { domain, name };
    match state.storage.get_observation_signals(&sref) {
        Ok(signals) => Json(SignalsSummary {
            discovery_count: signals.discovery_count,
            usage_count: signals.usage_count,
            flag_count: signals.flag_count,
            composition_count: signals.composition_count,
            last_activity: signals.last_activity,
        })
        .into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

// ─── Provenance & Versions ──────────────────────────────────────────────────

async fn get_provenance(
    State(state): State<S>,
    Path((domain, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let sref = ServiceRef { domain, name };
    match state.storage.get_provenance(&sref) {
        Ok(sources) => Json(serde_json::json!({"composed_from": sources})).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

async fn get_versions(
    State(state): State<S>,
    Path((domain, name)): Path<(String, String)>,
) -> impl IntoResponse {
    let sref = ServiceRef { domain, name };
    match state.storage.list_versions(&sref) {
        Ok(versions) => Json(serde_json::json!({"versions": versions})).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

// ─── Emergence ──────────────────────────────────────────────────────────────

#[derive(serde::Deserialize)]
struct PatternQuery {
    #[serde(default = "default_pattern_limit")]
    limit: u32,
}

fn default_pattern_limit() -> u32 {
    10
}

async fn get_patterns(
    State(state): State<S>,
    Query(params): Query<PatternQuery>,
) -> impl IntoResponse {
    match state.storage.get_top_patterns(params.limit) {
        Ok(patterns) => Json(serde_json::json!({"patterns": patterns})).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

async fn get_trending(State(state): State<S>) -> impl IntoResponse {
    // Simple: return services ordered by recent observation count
    let services = state
        .storage
        .list_services(&ServiceFilter {
            active_only: true,
            ..Default::default()
        })
        .unwrap_or_default();

    let mut scored: Vec<(ServiceRef, u64)> = services
        .iter()
        .filter_map(|svc| {
            let sref = ServiceRef {
                domain: svc.domain.clone(),
                name: svc.name.clone(),
            };
            let signals = state
                .storage
                .get_observation_signals(&sref)
                .ok()?;
            let total = signals.usage_count + signals.discovery_count;
            Some((sref, total))
        })
        .collect();

    scored.sort_by(|a, b| b.1.cmp(&a.1));
    scored.truncate(10);

    let trending: Vec<serde_json::Value> = scored
        .into_iter()
        .map(|(sref, count)| {
            serde_json::json!({
                "domain": sref.domain,
                "name": sref.name,
                "observation_count": count,
            })
        })
        .collect();

    Json(serde_json::json!({"trending": trending}))
}

// ─── Federation ─────────────────────────────────────────────────────────────

async fn list_peers(State(state): State<S>) -> impl IntoResponse {
    match state.storage.list_peers() {
        Ok(peers) => Json(serde_json::json!({"peers": peers})).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

async fn add_peer(
    State(state): State<S>,
    Json(peer): Json<PeerRecord>,
) -> impl IntoResponse {
    match state.storage.add_peer(&peer) {
        Ok(url) => (StatusCode::CREATED, Json(serde_json::json!({"status": "added", "url": url}))).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

async fn remove_peer(
    State(state): State<S>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // id is the peer URL
    match state.storage.remove_peer(&id) {
        Ok(()) => Json(serde_json::json!({"status": "removed"})).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

async fn sync_peer() -> impl IntoResponse {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(serde_json::json!({
            "error": "federation sync not implemented in reference server",
            "description": "A production implementation would: 1) GET /export from peer, 2) merge services by manifest_hash, 3) recompute local trust, 4) send local public services to peer"
        })),
    )
}

#[derive(serde::Deserialize)]
struct ExportQuery {
    since: Option<String>,
}

async fn export_services(
    State(state): State<S>,
    Query(params): Query<ExportQuery>,
) -> impl IntoResponse {
    match state
        .storage
        .export_public_services(params.since.as_deref())
    {
        Ok(services) => Json(services).into_response(),
        Err(e) => err(StatusCode::INTERNAL_SERVER_ERROR, &e.message),
    }
}

// ─── Admin ──────────────────────────────────────────────────────────────────

async fn recompute_trust(State(state): State<S>) -> impl IntoResponse {
    let services = state
        .storage
        .list_services(&ServiceFilter {
            active_only: true,
            ..Default::default()
        })
        .unwrap_or_default();
    let profiles = state.storage.list_profiles().unwrap_or_default();

    let mut count = 0;
    for svc in &services {
        let sref = ServiceRef {
            domain: svc.domain.clone(),
            name: svc.name.clone(),
        };
        for profile in &profiles {
            let signals = state
                .storage
                .get_observation_signals(&sref)
                .unwrap_or_default();
            let input = TrustInput {
                manifest: svc.manifest.clone(),
                profile: profile.clone(),
                signals,
            };
            let output = state.scorer.score(&input);
            let _ = state
                .storage
                .upsert_trust_score(&sref, &profile.name, &output);
            count += 1;
        }
    }

    Json(serde_json::json!({"status": "recomputed", "scores_updated": count}))
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn recompute_service_trust(state: &AppState, sref: &ServiceRef) {
    if let Ok(Some(svc)) = state.storage.get_service(sref) {
        let profiles = state.storage.list_profiles().unwrap_or_default();
        let signals = state
            .storage
            .get_observation_signals(sref)
            .unwrap_or_default();
        for profile in &profiles {
            let input = TrustInput {
                manifest: svc.manifest.clone(),
                profile: profile.clone(),
                signals: signals.clone(),
            };
            let output = state.scorer.score(&input);
            let _ = state
                .storage
                .upsert_trust_score(sref, &profile.name, &output);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extractor_json::JsonExtractor;
    use crate::identity_apikey::ApiKeyVerifier;
    use crate::matcher_sql::SqlMatcher;
    use crate::storage_sqlite::SqliteStorage;
    use crate::sync_stub::StubSync;
    use crate::trust_simple::SimpleScorer;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn test_app() -> Router {
        let state = AppState {
            storage: Box::new(SqliteStorage::open_in_memory().unwrap()),
            scorer: Box::new(SimpleScorer::new()),
            matcher: Box::new(SqlMatcher::new()),
            extractor: Box::new(JsonExtractor::new()),
            identity: Box::new(ApiKeyVerifier::new(None, None)),
            sync: Box::new(StubSync::new()),
            network_id: "test".into(),
            scope: "private".into(),
        };
        router(state)
    }

    async fn body_json(resp: axum::http::Response<Body>) -> serde_json::Value {
        let bytes = axum::body::to_bytes(resp.into_body(), 1024 * 1024)
            .await
            .unwrap();
        if bytes.is_empty() {
            return serde_json::json!(null);
        }
        serde_json::from_slice(&bytes).unwrap_or_else(|_| {
            serde_json::json!({"raw": String::from_utf8_lossy(&bytes).to_string()})
        })
    }

    fn register_body() -> String {
        serde_json::json!({
            "domain": "bakery.example.com",
            "manifest": {
                "name": "SweetCakes",
                "version": "0.1.0",
                "state": {"price": {"type": "number"}, "items": {"type": "list"}},
                "server_functions": ["order", "get_menu"],
                "actions": ["add_to_cart"],
                "data_sources": [{"name": "menu", "url": "https://bakery.example.com/api/menu", "type": "fetch"}]
            }
        })
        .to_string()
    }

    async fn register_bakery(app: &mut Router) -> serde_json::Value {
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/discovery/services")
            .header("content-type", "application/json")
            .body(Body::from(register_body()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        body_json(resp).await
    }

    #[tokio::test]
    async fn test_health() {
        let app = test_app();
        let req = Request::builder()
            .uri("/health")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_info() {
        let app = test_app();
        let req = Request::builder()
            .uri("/api/v1/discovery/info")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        assert_eq!(json["network_id"], "test");
        assert_eq!(json["services"], 0);
    }

    #[tokio::test]
    async fn test_register_service() {
        let mut app = test_app();
        let json = register_bakery(&mut app).await;
        assert_eq!(json["name"], "SweetCakes");
        assert!(json["trust_scores"]["default"].as_f64().unwrap() > 0.0);
        assert!(json["capabilities_indexed"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_get_service() {
        let app = test_app();
        // Register
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/discovery/services")
            .header("content-type", "application/json")
            .body(Body::from(register_body()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Verify info shows 1 service registered
        let req = Request::builder()
            .uri("/api/v1/discovery/info")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let info = body_json(resp).await;
        assert_eq!(info["services"], 1, "service should be registered");

        // Get
        let req = Request::builder()
            .uri("/api/v1/discovery/services/bakery.example.com/SweetCakes")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let status = resp.status();
        let json = body_json(resp).await;
        assert_eq!(status, StatusCode::OK, "GET returned {status}: {json:?}");
        assert_eq!(json["name"], "SweetCakes");
    }

    #[tokio::test]
    async fn test_get_service_not_found() {
        let app = test_app();
        let req = Request::builder()
            .uri("/api/v1/discovery/services/nonexistent.com/Nope")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_search_capabilities() {
        let mut app = test_app();
        register_bakery(&mut app).await;

        let query = serde_json::json!({
            "require": [{"kind": "server_function", "name": "order"}]
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/discovery/search")
            .header("content-type", "application/json")
            .body(Body::from(query.to_string()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        assert_eq!(json["total"], 1);
        assert_eq!(json["results"][0]["domain"], "bakery.example.com");
    }

    #[tokio::test]
    async fn test_search_no_results() {
        let mut app = test_app();
        register_bakery(&mut app).await;

        let query = serde_json::json!({
            "require": [{"kind": "server_function", "name": "nonexistent"}]
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/discovery/search")
            .header("content-type", "application/json")
            .body(Body::from(query.to_string()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let json = body_json(resp).await;
        assert_eq!(json["total"], 0);
    }

    #[tokio::test]
    async fn test_text_search() {
        let mut app = test_app();
        register_bakery(&mut app).await;

        let req = Request::builder()
            .uri("/api/v1/discovery/search?q=order")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        assert!(json["total"].as_u64().unwrap() >= 1);
    }

    #[tokio::test]
    async fn test_observe_and_signals() {
        let app = test_app();

        // Register
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/discovery/services")
            .header("content-type", "application/json")
            .body(Body::from(register_body()))
            .unwrap();
        app.clone().oneshot(req).await.unwrap();

        // Observe
        let obs = serde_json::json!({
            "kind": "usage",
            "service_domain": "bakery.example.com",
            "service_name": "SweetCakes",
            "payload": {"success": true}
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/discovery/observe")
            .header("content-type", "application/json")
            .body(Body::from(obs.to_string()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        // Check signals
        let req = Request::builder()
            .uri("/api/v1/discovery/services/bakery.example.com/SweetCakes/signals")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
        let json = body_json(resp).await;
        assert_eq!(json["usage_count"], 1);
    }

    #[tokio::test]
    async fn test_flag_service() {
        let mut app = test_app();
        register_bakery(&mut app).await;

        let flag = serde_json::json!({
            "service_domain": "bakery.example.com",
            "service_name": "SweetCakes",
            "reason": "manifest_mismatch",
            "evidence": "sends data to ad tracker"
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/discovery/flag")
            .header("content-type", "application/json")
            .body(Body::from(flag.to_string()))
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
        let json = body_json(resp).await;
        assert!(!json["deactivated"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_composition() {
        let mut app = test_app();
        register_bakery(&mut app).await;

        let comp = serde_json::json!({
            "services": [
                {"domain": "bakery.example.com", "name": "SweetCakes"},
                {"domain": "venue.com", "name": "Event Hall"}
            ]
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/discovery/compose")
            .header("content-type", "application/json")
            .body(Body::from(comp.to_string()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let req = Request::builder()
            .uri("/api/v1/discovery/patterns?limit=10")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let json = body_json(resp).await;
        assert_eq!(json["patterns"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_profiles() {
        let app = test_app();
        let req = Request::builder()
            .uri("/api/v1/discovery/profiles")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let json = body_json(resp).await;
        assert_eq!(json.as_array().unwrap().len(), 6);
    }

    #[tokio::test]
    async fn test_peers_crud() {
        let app = test_app();

        let peer = serde_json::json!({
            "url": "https://peer1.example.com",
            "name": "Peer 1",
            "active": true
        });
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/discovery/peers")
            .header("content-type", "application/json")
            .body(Body::from(peer.to_string()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);

        let req = Request::builder()
            .uri("/api/v1/discovery/peers")
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let json = body_json(resp).await;
        assert_eq!(json["peers"].as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_sync_returns_501() {
        let app = test_app();
        let req = Request::builder()
            .method("POST")
            .uri("/api/v1/discovery/peers/sync")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_IMPLEMENTED);
    }

    #[tokio::test]
    async fn test_export() {
        let mut app = test_app();
        register_bakery(&mut app).await;

        let req = Request::builder()
            .uri("/api/v1/discovery/export")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        let json = body_json(resp).await;
        assert_eq!(json.as_array().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_dashboard_serves_html() {
        let app = test_app();
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    // ─── Scenario Tests ─────────────────────────────────────────────────

    fn venue_body() -> String {
        serde_json::json!({
            "domain": "venue.example.com",
            "manifest": {
                "name": "EventHall",
                "version": "0.1.0",
                "state": {"capacity": {"type": "number"}, "location": {"type": "text"}},
                "server_functions": ["book"],
                "actions": ["reserve"]
            }
        }).to_string()
    }

    fn catering_body() -> String {
        serde_json::json!({
            "domain": "catering.example.com",
            "manifest": {
                "name": "CateringCo",
                "version": "0.1.0",
                "state": {"menu": {"type": "list"}, "price": {"type": "number"}},
                "server_functions": ["order_catering"],
                "actions": ["select_menu"]
            }
        }).to_string()
    }

    async fn post_json(app: &Router, uri: &str, body: &str) -> (StatusCode, serde_json::Value) {
        let req = Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let json = body_json(resp).await;
        (status, json)
    }

    async fn get_json(app: &Router, uri: &str) -> (StatusCode, serde_json::Value) {
        let req = Request::builder()
            .uri(uri)
            .body(Body::empty())
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let json = body_json(resp).await;
        (status, json)
    }

    /// Scenario 1: Full publish → discover → use lifecycle
    #[tokio::test]
    async fn scenario_publish_discover_use() {
        let app = test_app();

        // 1. Register bakery
        let (status, reg) = post_json(&app, "/api/v1/discovery/services", &register_body()).await;
        assert_eq!(status, StatusCode::CREATED);
        let initial_trust = reg["trust_scores"]["default"].as_f64().unwrap();
        assert!(initial_trust > 0.5);

        // 2. Search by capability → find it
        let query = serde_json::json!({"require": [{"kind": "server_function", "name": "order"}]});
        let (_, search) = post_json(&app, "/api/v1/discovery/search", &query.to_string()).await;
        assert_eq!(search["total"], 1);
        assert_eq!(search["results"][0]["domain"], "bakery.example.com");

        // 3. Record discovery + usage observations
        let obs = serde_json::json!({
            "kind": "discovery", "service_domain": "bakery.example.com",
            "service_name": "SweetCakes", "payload": {}
        });
        let (status, _) = post_json(&app, "/api/v1/discovery/observe", &obs.to_string()).await;
        assert_eq!(status, StatusCode::CREATED);

        let obs = serde_json::json!({
            "kind": "usage", "service_domain": "bakery.example.com",
            "service_name": "SweetCakes", "payload": {"success": true}
        });
        post_json(&app, "/api/v1/discovery/observe", &obs.to_string()).await;

        // 4. Verify signals
        let (_, signals) = get_json(&app, "/api/v1/discovery/services/bakery.example.com/SweetCakes/signals").await;
        assert_eq!(signals["usage_count"], 1);
        assert_eq!(signals["discovery_count"], 1);
    }

    /// Scenario 2: Compose → provenance → trust cascade on flag
    #[tokio::test]
    async fn scenario_composition_provenance_trust() {
        let app = test_app();

        // 1. Register 3 services
        post_json(&app, "/api/v1/discovery/services", &register_body()).await;
        post_json(&app, "/api/v1/discovery/services", &venue_body()).await;
        post_json(&app, "/api/v1/discovery/services", &catering_body()).await;

        // 2. Register composed "party planner"
        let composed = serde_json::json!({
            "domain": "agent.local",
            "manifest": {
                "name": "PartyPlanner",
                "state": {"plan": {"type": "text"}},
                "server_functions": ["plan_party"]
            },
            "publisher": "agent:claude-test",
            "composed_from": [
                {"domain": "bakery.example.com", "name": "SweetCakes"},
                {"domain": "venue.example.com", "name": "EventHall"},
                {"domain": "catering.example.com", "name": "CateringCo"}
            ]
        });
        let (status, _) = post_json(&app, "/api/v1/discovery/services", &composed.to_string()).await;
        assert_eq!(status, StatusCode::CREATED);

        // 3. Verify provenance
        let (_, prov) = get_json(&app, "/api/v1/discovery/services/agent.local/PartyPlanner/provenance").await;
        assert_eq!(prov["composed_from"].as_array().unwrap().len(), 3);

        // 4. Record composition pattern
        let comp = serde_json::json!({
            "services": [
                {"domain": "bakery.example.com", "name": "SweetCakes"},
                {"domain": "venue.example.com", "name": "EventHall"},
                {"domain": "catering.example.com", "name": "CateringCo"}
            ]
        });
        post_json(&app, "/api/v1/discovery/compose", &comp.to_string()).await;

        let (_, patterns) = get_json(&app, "/api/v1/discovery/patterns?limit=10").await;
        assert_eq!(patterns["patterns"].as_array().unwrap().len(), 1);
    }

    /// Scenario 3: Flag → deactivation after threshold
    #[tokio::test]
    async fn scenario_flag_deactivation() {
        let app = test_app();
        post_json(&app, "/api/v1/discovery/services", &register_body()).await;

        // Flag 5 times (threshold)
        for i in 0..5 {
            let flag = serde_json::json!({
                "service_domain": "bakery.example.com",
                "service_name": "SweetCakes",
                "reason": "test_flag",
                "agent_id": format!("agent:{}", i)
            });
            let (_, resp) = post_json(&app, "/api/v1/discovery/flag", &flag.to_string()).await;
            if i == 4 {
                assert!(resp["deactivated"].as_bool().unwrap(), "should be deactivated after 5 flags");
            }
        }

        // Service should no longer appear in search
        let query = serde_json::json!({"require": [{"kind": "server_function", "name": "order"}]});
        let (_, search) = post_json(&app, "/api/v1/discovery/search", &query.to_string()).await;
        assert_eq!(search["total"], 0, "deactivated service should not appear in search");

        // But should still be retrievable directly (with active=false)
        let (status, svc) = get_json(&app, "/api/v1/discovery/services/bakery.example.com/SweetCakes").await;
        assert_eq!(status, StatusCode::OK);
        assert!(!svc["active"].as_bool().unwrap());
    }

    /// Scenario 4: Version update preserves history
    #[tokio::test]
    async fn scenario_version_history() {
        let app = test_app();

        // Register v0.1.0
        post_json(&app, "/api/v1/discovery/services", &register_body()).await;

        // Update to v0.2.0
        let v2 = serde_json::json!({
            "domain": "bakery.example.com",
            "manifest": {
                "name": "SweetCakes",
                "version": "0.2.0",
                "state": {"price": {"type": "number"}, "items": {"type": "list"}, "specials": {"type": "list"}},
                "server_functions": ["order", "get_menu", "get_specials"],
                "actions": ["add_to_cart"]
            }
        });
        post_json(&app, "/api/v1/discovery/services", &v2.to_string()).await;

        // Current version should be 0.2.0
        let (_, svc) = get_json(&app, "/api/v1/discovery/services/bakery.example.com/SweetCakes").await;
        assert_eq!(svc["version"], "0.2.0");

        // Version history should show 0.1.0
        let (_, versions) = get_json(&app, "/api/v1/discovery/services/bakery.example.com/SweetCakes/versions").await;
        let v_list = versions["versions"].as_array().unwrap();
        assert!(!v_list.is_empty(), "should have version history");
        assert!(v_list.iter().any(|v| v["version"] == "0.1.0"), "should contain v0.1.0");
    }

    /// Scenario 5: Pattern emergence from repeated compositions
    #[tokio::test]
    async fn scenario_pattern_emergence() {
        let app = test_app();

        let a = ServiceRef { domain: "a.com".into(), name: "A".into() };
        let b = ServiceRef { domain: "b.com".into(), name: "B".into() };
        let c = ServiceRef { domain: "c.com".into(), name: "C".into() };

        // Compose {A, B} many times
        let comp_ab = serde_json::json!({"services": [{"domain":"a.com","name":"A"},{"domain":"b.com","name":"B"}]});
        for _ in 0..10 {
            post_json(&app, "/api/v1/discovery/compose", &comp_ab.to_string()).await;
        }

        // Compose {A, C} fewer times
        let comp_ac = serde_json::json!({"services": [{"domain":"a.com","name":"A"},{"domain":"c.com","name":"C"}]});
        for _ in 0..3 {
            post_json(&app, "/api/v1/discovery/compose", &comp_ac.to_string()).await;
        }

        // Patterns should be ordered by frequency
        let (_, patterns) = get_json(&app, "/api/v1/discovery/patterns?limit=10").await;
        let p = patterns["patterns"].as_array().unwrap();
        assert_eq!(p.len(), 2);
        assert_eq!(p[0]["frequency"], 10, "most frequent pattern first");
        assert_eq!(p[1]["frequency"], 3);
    }

    /// Scenario 6: Visibility — internal services don't export
    #[tokio::test]
    async fn scenario_visibility_export() {
        let app = test_app();

        // Public service
        post_json(&app, "/api/v1/discovery/services", &register_body()).await;

        // Internal service
        let internal = serde_json::json!({
            "domain": "internal.corp",
            "manifest": {"name": "Payroll", "state": {"salary": {"type": "number"}}},
            "visibility": "internal"
        });
        post_json(&app, "/api/v1/discovery/services", &internal.to_string()).await;

        // Info should show 2 services
        let (_, info) = get_json(&app, "/api/v1/discovery/info").await;
        assert_eq!(info["services"], 2);

        // Export should only include the public one
        let (_, exported) = get_json(&app, "/api/v1/discovery/export").await;
        assert_eq!(exported.as_array().unwrap().len(), 1);
        assert_eq!(exported[0]["domain"], "bakery.example.com");
    }

    /// Scenario 7: Trust differentiation — clean vs risky manifests
    #[tokio::test]
    async fn scenario_trust_differentiation() {
        let app = test_app();

        // Clean bakery
        post_json(&app, "/api/v1/discovery/services", &register_body()).await;

        // Risky tracking service
        let risky = serde_json::json!({
            "domain": "sketchy.example.com",
            "manifest": {
                "name": "SketchyTracker",
                "state": {
                    "email": {"type": "text"},
                    "phone": {"type": "text"},
                    "ssn": {"type": "text"},
                    "credit_card": {"type": "text"}
                },
                "server_functions": ["submit", "track", "report"],
                "data_sources": [
                    {"name": "ads", "url": "https://adtracker1.com/pixel", "type": "fetch"},
                    {"name": "analytics", "url": "https://analytics2.com/track", "type": "fetch"},
                    {"name": "more_ads", "url": "https://adnetwork3.com/beacon", "type": "fetch"}
                ]
            }
        });
        post_json(&app, "/api/v1/discovery/services", &risky.to_string()).await;

        // Get trust scores
        let (_, bakery_trust) = get_json(&app, "/api/v1/discovery/services/bakery.example.com/SweetCakes/trust").await;
        let (_, risky_trust) = get_json(&app, "/api/v1/discovery/services/sketchy.example.com/SketchyTracker/trust").await;

        let bakery_score = bakery_trust["scores"]["default"]["score"].as_f64().unwrap();
        let risky_score = risky_trust["scores"]["default"]["score"].as_f64().unwrap();

        assert!(bakery_score > risky_score,
            "clean bakery ({bakery_score}) should score higher than risky tracker ({risky_score})");
        assert!(bakery_score > 0.7, "bakery should score > 0.7, got {bakery_score}");
        assert!(risky_score < 0.6, "risky tracker should score < 0.6, got {risky_score}");
    }
}
