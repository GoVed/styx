use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(get_telemetry))
}

async fn get_telemetry(State(state): State<AppState>) -> impl IntoResponse {
    let containers = state.docker.list_containers(false).await.unwrap_or_default();
    let mcp_tools = state.mcp.list_tools().await;
    let pending = state.hitl.list_pending_tickets().await.len();
    let active_model = state
        .db
        .get_active_model_config()
        .await
        .ok()
        .flatten()
        .map(|m| format!("{}: {}", m.name, m.model_id))
        .unwrap_or_else(|| "Local vLLM (Docker)".to_string());

    let queue_status = state.turn_queue.get_status().await;

    let snapshot = state
        .telemetry
        .collect_snapshot(
            containers.len(),
            mcp_tools.len(),
            pending,
            active_model,
            queue_status.queued_count,
            queue_status.active_count,
            queue_status.max_concurrent_turns,
        )
        .await;

    Json(snapshot)
}
