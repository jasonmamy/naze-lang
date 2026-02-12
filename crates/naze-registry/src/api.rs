use std::sync::Arc;

use axum::{
    extract::{DefaultBodyLimit, Multipart, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tower_http::cors::CorsLayer;

use crate::db::Db;
use crate::storage::Storage;

pub struct AppState {
    pub db: Db,
    pub storage: Storage,
}

pub fn router(db: Db, storage: Storage) -> Router {
    let state = Arc::new(AppState { db, storage });

    Router::new()
        .route("/health", get(health))
        .route("/api/v1/packages", post(publish_package))
        .route("/api/v1/packages/*path", get(get_package_handler))
        .route("/api/v1/search", get(search_handler))
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024)) // 50MB
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    #[serde(default = "default_limit")]
    limit: u32,
}

fn default_limit() -> u32 {
    20
}

#[derive(Serialize)]
struct SearchResponse {
    packages: Vec<PackageInfo>,
}

#[derive(Serialize)]
struct PackageInfo {
    name: String,
    description: String,
    latest_version: String,
}

#[derive(Serialize)]
struct PackageDetail {
    name: String,
    description: String,
    latest_version: String,
    versions: Vec<VersionInfo>,
}

#[derive(Serialize)]
struct VersionInfo {
    version: String,
    checksum: String,
    naze_files: i64,
    created_at: String,
}

#[derive(Deserialize)]
struct PublishMetadata {
    name: String,
    version: String,
    #[serde(default)]
    description: String,
}

async fn search_handler(
    State(state): State<Arc<AppState>>,
    Query(query): Query<SearchQuery>,
) -> impl IntoResponse {
    match state.db.search(&query.q, query.limit) {
        Ok(packages) => {
            let response = SearchResponse {
                packages: packages
                    .into_iter()
                    .map(|p| PackageInfo {
                        name: p.name,
                        description: p.description,
                        latest_version: p.latest_version,
                    })
                    .collect(),
            };
            (StatusCode::OK, Json(serde_json::json!(response))).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/v1/packages/{*path}
/// Handles: `name`, `name/versions`, `name/version`, `name/version/download`,
/// and scoped: `@scope/name`, `@scope/name/versions`, etc.
async fn get_package_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    let parts: Vec<&str> = path.split('/').collect();

    // Parse the package name and remaining path segments
    let (pkg_name, rest) = if parts.first().map_or(false, |p| p.starts_with('@')) {
        // Scoped package: @scope/name/...
        if parts.len() < 2 {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "invalid scoped package name"})),
            )
                .into_response();
        }
        let name = format!("{}/{}", parts[0], parts[1]);
        (name, &parts[2..])
    } else {
        (parts[0].to_string(), &parts[1..])
    };

    // Look up the package
    let pkg = match state.db.get_package(&pkg_name) {
        Ok(Some(p)) => p,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "package not found"})),
            )
                .into_response();
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    match rest {
        // GET /api/v1/packages/name — package detail with all versions
        [] => {
            let versions = state.db.get_versions(pkg.id).unwrap_or_default();
            let detail = PackageDetail {
                name: pkg.name,
                description: pkg.description,
                latest_version: pkg.latest_version,
                versions: versions
                    .into_iter()
                    .map(|v| VersionInfo {
                        version: v.version,
                        checksum: v.checksum,
                        naze_files: v.naze_files,
                        created_at: v.created_at,
                    })
                    .collect(),
            };
            (StatusCode::OK, Json(serde_json::json!(detail))).into_response()
        }
        // GET /api/v1/packages/name/versions — list versions only
        ["versions"] => {
            let versions = state.db.get_versions(pkg.id).unwrap_or_default();
            let version_list: Vec<VersionInfo> = versions
                .into_iter()
                .map(|v| VersionInfo {
                    version: v.version,
                    checksum: v.checksum,
                    naze_files: v.naze_files,
                    created_at: v.created_at,
                })
                .collect();
            (StatusCode::OK, Json(serde_json::json!({"versions": version_list}))).into_response()
        }
        // GET /api/v1/packages/name/version — specific version info
        [version] => match state.db.get_version(pkg.id, version) {
            Ok(Some(v)) => (
                StatusCode::OK,
                Json(serde_json::json!({
                    "name": pkg.name,
                    "version": v.version,
                    "checksum": v.checksum,
                    "naze_files": v.naze_files,
                    "created_at": v.created_at,
                })),
            )
                .into_response(),
            Ok(None) => (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "version not found"})),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response(),
        },
        // GET /api/v1/packages/name/version/download — download tarball
        [version, "download"] => {
            match state.db.get_version(pkg.id, version) {
                Ok(Some(_v)) => match state.storage.get_tarball(&pkg_name, version) {
                    Ok(data) => (
                        StatusCode::OK,
                        [
                            ("content-type", "application/gzip"),
                            (
                                "content-disposition",
                                &format!(
                                    "attachment; filename=\"{}-{}.tar.gz\"",
                                    pkg_name.replace('/', "__"),
                                    version
                                ),
                            ),
                        ],
                        data,
                    )
                        .into_response(),
                    Err(e) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": e.to_string()})),
                    )
                        .into_response(),
                },
                Ok(None) => (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "version not found"})),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response(),
            }
        }
        _ => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "invalid path"})),
        )
            .into_response(),
    }
}

/// POST /api/v1/packages — publish a package
/// Expects multipart form: "metadata" (JSON) + "tarball" (binary)
async fn publish_package(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut metadata: Option<PublishMetadata> = None;
    let mut tarball_data: Option<Vec<u8>> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        match field.name() {
            Some("metadata") => {
                let text = field.text().await.unwrap_or_default();
                match serde_json::from_str::<PublishMetadata>(&text) {
                    Ok(m) => metadata = Some(m),
                    Err(e) => {
                        return (
                            StatusCode::BAD_REQUEST,
                            Json(serde_json::json!({"error": format!("invalid metadata: {e}")})),
                        )
                            .into_response();
                    }
                }
            }
            Some("tarball") => {
                tarball_data = field.bytes().await.ok().map(|b| b.to_vec());
            }
            _ => {}
        }
    }

    let Some(meta) = metadata else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "missing metadata field"})),
        )
            .into_response();
    };
    let Some(tarball) = tarball_data else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "missing tarball field"})),
        )
            .into_response();
    };

    // Compute checksum
    let checksum = format!("{:x}", Sha256::digest(&tarball));

    // Count .naze files in tarball
    let naze_files = count_naze_files(&tarball).unwrap_or(0);

    // Ensure package exists (create if new)
    let pkg_id = match state.db.get_package(&meta.name) {
        Ok(Some(pkg)) => pkg.id,
        Ok(None) => match state.db.insert_package(&meta.name, &meta.description) {
            Ok(id) => id,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": e.to_string()})),
                )
                    .into_response();
            }
        },
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    // Store tarball
    let tarball_path = match state.storage.store_tarball(&meta.name, &meta.version, &tarball) {
        Ok(p) => p,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        }
    };

    // Insert version
    if let Err(e) = state.db.insert_version(
        pkg_id,
        &meta.version,
        &checksum,
        &tarball_path,
        naze_files,
    ) {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({"error": format!("version already exists: {e}")})),
        )
            .into_response();
    }

    // Update latest version
    let _ = state.db.update_latest_version(pkg_id, &meta.version);

    (
        StatusCode::CREATED,
        Json(serde_json::json!({
            "name": meta.name,
            "version": meta.version,
            "checksum": checksum,
            "naze_files": naze_files,
        })),
    )
        .into_response()
}

fn count_naze_files(tarball: &[u8]) -> Result<i64, Box<dyn std::error::Error>> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let decoder = GzDecoder::new(tarball);
    let mut archive = Archive::new(decoder);
    let mut count = 0i64;
    for entry in archive.entries()? {
        let entry = entry?;
        if let Some(name) = entry.path()?.to_str() {
            if name.ends_with(".naze") {
                count += 1;
            }
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    fn test_app() -> Router {
        let db = Db::open_in_memory().unwrap();
        db.init_schema().unwrap();
        let tmp = std::env::temp_dir().join("naze-reg-api-test");
        let _ = std::fs::remove_dir_all(&tmp);
        let storage = Storage::new(tmp.to_str().unwrap());
        router(db, storage)
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
    async fn test_search_empty() {
        let app = test_app();
        let req = Request::builder()
            .uri("/api/v1/search?q=test")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_package_not_found() {
        let app = test_app();
        let req = Request::builder()
            .uri("/api/v1/packages/nonexistent")
            .body(Body::empty())
            .unwrap();
        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }
}
