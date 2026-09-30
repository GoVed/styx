use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode, Uri},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::warn;

use crate::state::AppState;

/// Helper to extract bearer token or X-Styx-Access-Key or query ?token=
pub async fn verify_token_from_parts(state: &AppState, headers: &HeaderMap, uri: &Uri) -> bool {
    // If auth is not initialized, do NOT allow access to protected endpoints!
    if !state.db.is_auth_initialized().await.unwrap_or(false) {
        return false;
    }

    // 1. Check Authorization: Bearer <token>
    if let Some(auth_hdr) = headers.get(header::AUTHORIZATION).and_then(|h| h.to_str().ok()) {
        let trimmed = auth_hdr.trim();
        let token = if trimmed.starts_with("Bearer ") || trimmed.starts_with("bearer ") {
            &trimmed[7..]
        } else {
            trimmed
        };
        if state.db.verify_access_key(token).await.unwrap_or(false) {
            return true;
        }
    }

    // 2. Check X-Styx-Access-Key: <key>
    if let Some(key_hdr) = headers.get("X-Styx-Access-Key").and_then(|h| h.to_str().ok())
        && state.db.verify_access_key(key_hdr).await.unwrap_or(false) {
            return true;
        }

    // 3. Check query param ?token=
    if let Some(uri_query) = uri.query() {
        for pair in uri_query.split('&') {
            let mut parts = pair.split('=');
            if let (Some(k), Some(v)) = (parts.next(), parts.next())
                && k == "token" && state.db.verify_access_key(v).await.unwrap_or(false) {
                    return true;
                }
        }
    }

    false
}

/// Middleware to enforce authentication on protected endpoints
pub async fn auth_middleware(
    State(state): State<AppState>,
    req: axum::extract::Request,
    next: Next,
) -> Response {
    let path = req.uri().path();

    // Whitelist public / auth endpoints
    if path == "/api/auth/status"
        || path == "/api/auth/setup"
        || path == "/api/auth/login"
    {
        return next.run(req).await;
    }

    // Whitelist static frontend assets (HTML, CSS, JS, favicon, etc.)
    // Note: NEVER whitelist /ws or /api!
    if !path.starts_with("/api") && !path.starts_with("/ws") {
        return next.run(req).await;
    }

    // If device is uninitialized, reject access to protected routes
    let is_initialized = state.db.is_auth_initialized().await.unwrap_or(false);
    if !is_initialized {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "error": "Device is not initialized. Please complete initial setup at /api/auth/setup before accessing protected resources."
            })),
        )
            .into_response();
    }

    let headers = req.headers().clone();
    let uri = req.uri().clone();

    if verify_token_from_parts(&state, &headers, &uri).await {
        next.run(req).await
    } else {
        warn!("Unauthorized access attempt to: {}", path);
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "error": "Unauthorized: Device Access Key required"
            })),
        )
            .into_response()
    }
}
