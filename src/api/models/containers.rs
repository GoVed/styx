use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::docker::engines::DeployModelRequest;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct LogsQuery {
    pub tail: Option<usize>,
}

pub async fn list_containers(State(state): State<AppState>) -> impl IntoResponse {
    match state.docker.list_containers(true).await {
        Ok(containers) => (
            StatusCode::OK,
            Json(json!({ "success": true, "containers": containers })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

pub async fn deploy_container(
    State(state): State<AppState>,
    Json(payload): Json<DeployModelRequest>,
) -> impl IntoResponse {
    match state.docker.deploy_model_container(&payload).await {
        Ok(container_id) => {
            let ctx = payload.context_window.unwrap_or(32768) as i64;
            let provider = match payload.engine {
                crate::docker::engines::EngineKind::Vllm => "docker_vllm",
                crate::docker::engines::EngineKind::LlamaCpp => "docker_llamacpp",
                crate::docker::engines::EngineKind::Ollama => "docker_ollama",
            };
            let port = payload.default_port();
            let base_url = format!("http://localhost:{}/v1", port);
            let display_name = format!(
                "Local {} ({})",
                match payload.engine {
                    crate::docker::engines::EngineKind::Vllm => "vLLM",
                    crate::docker::engines::EngineKind::LlamaCpp => "llama.cpp",
                    crate::docker::engines::EngineKind::Ollama => "Ollama",
                },
                payload.name
            );
            let model_id = payload.hf_repo.clone();

            let existing_configs = state.db.list_model_configs().await.unwrap_or_default();
            let existing = existing_configs
                .into_iter()
                .find(|c| c.base_url.as_deref() == Some(&base_url) || c.model_id == model_id);

            let target_id = if let Some(found) = existing {
                let _ = state
                    .db
                    .update_model_config(&found.id, &display_name, &model_id, ctx)
                    .await;
                found.id
            } else {
                match state
                    .db
                    .add_model_config(
                        &display_name,
                        provider,
                        Some(&base_url),
                        None,
                        &model_id,
                        ctx,
                        None,
                    )
                    .await
                {
                    Ok(rec) => rec.id,
                    Err(_) => String::new(),
                }
            };

            if !target_id.is_empty() {
                let _ = state.db.set_active_model(&target_id).await;
            }

            (
                StatusCode::CREATED,
                Json(json!({
                    "success": true,
                    "container_id": container_id,
                    "name": payload.name,
                    "port": port,
                    "context_length": ctx,
                    "config_id": target_id
                })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

pub async fn start_container(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.docker.start_container(&id).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "success": true }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

pub async fn stop_container(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.docker.stop_container(&id).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "success": true }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

pub async fn restart_container(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.docker.restart_container(&id).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "success": true }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

pub async fn delete_container(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.docker.delete_container(&id).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "success": true }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

pub async fn get_container_logs(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(q): Query<LogsQuery>,
) -> impl IntoResponse {
    let tail = q.tail.unwrap_or(200);
    match state.docker.get_container_logs(&id, tail).await {
        Ok(logs) => (StatusCode::OK, Json(json!({ "success": true, "logs": logs }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}
