use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::hitl::ApprovalDecision;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tickets", get(list_tickets))
        .route("/decide", post(submit_decision))
}

#[derive(Deserialize)]
struct TicketsQuery {
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct DecideRequest {
    ticket_id: String,
    action: String, // "APPROVE", "REJECT", "MODIFY"
    reason: Option<String>,
    modified_arguments: Option<Value>,
}

async fn list_tickets(
    State(state): State<AppState>,
    Query(query): Query<TicketsQuery>,
) -> impl IntoResponse {
    let limit = query.limit.unwrap_or(50);
    let pending_mem = state.hitl.list_pending_tickets().await;
    let history = state.db.list_approval_tickets(limit).await.unwrap_or_default();

    Json(json!({
        "success": true,
        "pending": pending_mem,
        "history": history
    }))
}

async fn submit_decision(
    State(state): State<AppState>,
    Json(payload): Json<DecideRequest>,
) -> impl IntoResponse {
    let decision = match payload.action.to_uppercase().as_str() {
        "APPROVE" | "APPROVED" => ApprovalDecision::Approve,
        "REJECT" | "REJECTED" => ApprovalDecision::Reject {
            reason: payload.reason,
        },
        "MODIFY" | "MODIFY_PAYLOAD" => {
            if let Some(new_args) = payload.modified_arguments {
                ApprovalDecision::ModifyPayload {
                    new_arguments: new_args,
                }
            } else {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "success": false, "error": "modified_arguments required for MODIFY action" })),
                );
            }
        }
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid action. Must be APPROVE, REJECT, or MODIFY" })),
            );
        }
    };

    match state.hitl.resolve_approval(&payload.ticket_id, decision).await {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({ "success": true, "ticket_id": payload.ticket_id })),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}
