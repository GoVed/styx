use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/events", get(list_audit_events))
}

#[derive(Deserialize)]
struct AuditQuery {
    limit: Option<usize>,
}

async fn list_audit_events(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(100);
    match state.db.list_audit_events(limit).await {
        Ok(events) => (
            StatusCode::OK,
            Json(json!({ "success": true, "events": events })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}
