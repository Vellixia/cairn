//! Self-hosted pack registry (v0.5.0 Sprint 13).
//!
//! Stores published `.cairnpkg` tarballs under `<data_dir>/registry/packs/<name>/<version>/`,
//! along with a JSON metadata index (`<data_dir>/registry/index.json`) that's fast to scan
//! without unpacking. Optionally verifies the pack's Ed25519 signature against a set of
//! trusted public keys at publish time, so a tampered pack never lands on disk.
//!
//! ## Endpoints (mounted under `/registry`)
//!
//! - `POST   /registry/packs` — upload a tarball; verify Ed25519 if any trusted keys
//!   match; return a [`PublishReceipt`].
//! - `GET    /registry/packs` — list all pack metadata (newest first).
//! - `GET    /registry/packs/:name` — list versions of a single pack.
//! - `GET    /registry/packs/:name/:version/download` — stream the tarball.
//! - `DELETE /registry/packs/:name/:version` — unpublish (Sprint 14 revocation uses this).
//! - `GET    /registry/search?q=...` — substring search on name + description.
//!
//! ## Storage layout
//!
//! ```text
//! <data_dir>/registry/
//!   index.json                       — JSON array of PackMeta
//!   trusted_keys.json                — list of trusted author public keys
//!   revocations.jsonl                — append-only log of unpublish events
//!   packs/<name>/<version>.cairnpkg  — the actual tarball
//!   packs/<name>/<version>.manifest.json — cached manifest for fast metadata
//! ```
//!
//! The index file is rewritten on every mutation. For the v0.5.0 scale (hundreds of
//! packs, dozens of installs per day) that's fine; if it ever becomes the bottleneck,
//! move to a sqlite-backed metadata store without changing the public API.

pub mod store;

pub use store::{
    PackMeta, PublishReceipt, PublishStatus, Registry, RegistryError, RevocationEvent,
};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get},
    Json, Router,
};
use serde::Deserialize;
use std::sync::Arc;

/// Build the axum router. Mount under `/registry` so the rest of the cairn API keeps
/// its `/api/...` shape.
pub fn router(registry: Arc<Registry>) -> Router {
    Router::new()
        .route("/packs", get(list_packs).post(publish_pack))
        .route("/packs/:name", get(list_versions))
        .route("/packs/:name/:version/download", get(download_pack))
        .route("/packs/:name/:version", delete(revoke_pack))
        .route("/search", get(search_packs))
        .route("/trusted-keys", get(list_trusted_keys))
        .route("/revocations", get(list_revocations))
        .with_state(registry)
}

/// Registry-facing JSON error.
#[derive(Debug, serde::Serialize)]
pub struct RegistryApiError {
    pub error: String,
}

impl From<RegistryError> for RegistryApiError {
    fn from(e: RegistryError) -> Self {
        Self {
            error: e.to_string(),
        }
    }
}

/// `GET /registry/packs` — list all packs (newest first).
async fn list_packs(State(reg): State<Arc<Registry>>) -> Result<Json<Vec<PackMeta>>, Response> {
    reg.list_all().map(Json).map_err(|e| e.into_response())
}

/// `POST /registry/packs` — body is the raw tarball bytes. Optional `?trusted=<hex-pubkey>`
/// query string restricts which author keys are accepted for this publish. The response
/// is a [`PublishReceipt`] describing the verification + storage outcome.
async fn publish_pack(
    State(reg): State<Arc<Registry>>,
    Query(q): Query<PublishQuery>,
    body: axum::body::Bytes,
) -> Result<(StatusCode, Json<PublishReceipt>), Response> {
    match reg.publish(&body, q.trusted.as_deref()) {
        Ok(r) => Ok((StatusCode::CREATED, Json(r))),
        Err(RegistryError::InvalidSignature) => Err((
            StatusCode::UNAUTHORIZED,
            Json(RegistryApiError {
                error: "Ed25519 signature did not match any trusted key".into(),
            }),
        )
            .into_response()),
        Err(e) => Err((StatusCode::BAD_REQUEST, Json(RegistryApiError::from(e))).into_response()),
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct PublishQuery {
    /// Hex-encoded trusted public key to whitelist for this publish. If absent, the
    /// registry's configured `trusted_keys.json` list is used.
    pub trusted: Option<String>,
}

/// `GET /registry/packs/:name` — list versions of one pack.
async fn list_versions(
    State(reg): State<Arc<Registry>>,
    Path(name): Path<String>,
) -> Result<Json<Vec<PackMeta>>, Response> {
    reg.list_versions(&name)
        .map(Json)
        .map_err(|e| e.into_response())
}

/// `GET /registry/packs/:name/:version/download` — stream the tarball.
async fn download_pack(
    State(reg): State<Arc<Registry>>,
    Path((name, version)): Path<(String, String)>,
) -> Result<Response, Response> {
    let bytes = reg
        .download_bytes(&name, &version)
        .map_err(|e| e.into_response())?;
    Ok((
        StatusCode::OK,
        [(axum::http::header::CONTENT_TYPE, cairn_pack::MIME)],
        bytes,
    )
        .into_response())
}

/// `DELETE /registry/packs/:name/:version` — unpublish. Appends to the revocations log
/// so federation peers see the change in their next sync (Sprint 14).
async fn revoke_pack(
    State(reg): State<Arc<Registry>>,
    Path((name, version)): Path<(String, String)>,
) -> Result<Json<RevocationEvent>, Response> {
    reg.revoke(&name, &version)
        .map(Json)
        .map_err(|e| e.into_response())
}

#[derive(Debug, Default, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
}

/// `GET /registry/search?q=...` — case-insensitive substring search on `name + description`.
async fn search_packs(
    State(reg): State<Arc<Registry>>,
    Query(q): Query<SearchQuery>,
) -> Result<Json<Vec<PackMeta>>, Response> {
    let q = q.q.unwrap_or_default();
    reg.search(&q).map(Json).map_err(|e| e.into_response())
}

/// `GET /registry/trusted-keys` — list the trusted author public keys this registry will
/// accept signatures from.
async fn list_trusted_keys(
    State(reg): State<Arc<Registry>>,
) -> Result<Json<Vec<cairn_pack::PublicKey>>, Response> {
    reg.trusted_keys().map(Json).map_err(|e| e.into_response())
}

/// `GET /registry/revocations` — the append-only log of unpublish events.
async fn list_revocations(
    State(reg): State<Arc<Registry>>,
) -> Result<Json<Vec<RevocationEvent>>, Response> {
    reg.list_revocations()
        .map(Json)
        .map_err(|e| e.into_response())
}

/// Convenience impl so axum can convert a [`RegistryError`] into a response.
trait IntoResponseExt {
    fn into_response(self) -> Response;
}

impl IntoResponseExt for RegistryError {
    fn into_response(self) -> Response {
        let status = match &self {
            RegistryError::NotFound(_) => StatusCode::NOT_FOUND,
            RegistryError::AlreadyExists(_) => StatusCode::CONFLICT,
            RegistryError::InvalidSignature => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (
            status,
            Json(RegistryApiError {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}
