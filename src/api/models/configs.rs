use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::state::AppState;

#[derive(Deserialize)]
pub struct AddConfigRequest {
    pub name: String,
    pub provider: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model_id: String,
    pub context_length: Option<i64>,
}

#[derive(Deserialize)]
pub struct TestConnectionRequest {
    pub provider: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model_id: String,
}

#[allow(clippy::collapsible_if)]
pub async fn auto_sync_active_local_model(db: &crate::db::Database) {
    if let Ok(Some(active)) = db.get_active_model_config().await {
        if (active.provider == "docker_vllm"
            || active.provider == "docker_llamacpp"
            || active.provider == "docker_ollama")
            && let Some(base_url) = &active.base_url {
                let mut probe_url = format!("{}/models", base_url.trim_end_matches('/'));
                if std::path::Path::new("/.dockerenv").exists() {
                    probe_url = probe_url
                        .replace("http://localhost:", "http://host.docker.internal:")
                        .replace("http://127.0.0.1:", "http://host.docker.internal:");
                }

                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_millis(1000))
                    .build()
                    .unwrap_or_default();

                if let Ok(resp) = client.get(&probe_url).send().await
                    && resp.status().is_success()
                        && let Ok(body) = resp.json::<serde_json::Value>().await
                            && let Some(data) = body.get("data").and_then(|d| d.as_array())
                                && let Some(first_model) = data.first()
                                    && let Some(id_str) =
                                        first_model.get("id").and_then(|i| i.as_str())
                                    {
                                        let detected_ctx = first_model
                                            .get("max_model_len")
                                            .and_then(|l| l.as_i64())
                                            .unwrap_or(active.context_length);
                                        if id_str != active.model_id
                                            || (detected_ctx > 0
                                                && detected_ctx != active.context_length)
                                        {
                                            tracing::info!(
                                                "Auto-syncing active model: detected running engine model '{}' with context {} (was '{}' with context {})",
                                                id_str,
                                                detected_ctx,
                                                active.model_id,
                                                active.context_length
                                            );
                                            let new_name = format!(
                                                "Local {} ({})",
                                                if active.provider == "docker_vllm" {
                                                    "vLLM"
                                                } else if active.provider == "docker_llamacpp" {
                                                    "llama.cpp"
                                                } else {
                                                    "Ollama"
                                                },
                                                id_str
                                            );
                                            let _ = db
                                                .update_model_config(
                                                    &active.id,
                                                    &new_name,
                                                    id_str,
                                                    detected_ctx,
                                                )
                                                .await;
                                        }
                                    }
            }
    }
}

pub fn mask_api_key(key: Option<&str>) -> Option<String> {
    match key {
        Some(k) if !k.trim().is_empty() => {
            let trimmed = k.trim();
            if trimmed.len() > 8 {
                let prefix = &trimmed[..3.min(trimmed.len())];
                let suffix = &trimmed[trimmed.len().saturating_sub(4)..];
                Some(format!("{}...{}", prefix, suffix))
            } else {
                Some("********".to_string())
            }
        }
        _ => None,
    }
}

pub async fn list_model_configs(State(state): State<AppState>) -> impl IntoResponse {
    auto_sync_active_local_model(&state.db).await;

    match state.db.list_model_configs().await {
        Ok(configs) => {
            let masked_configs: Vec<_> = configs
                .into_iter()
                .map(|mut c| {
                    c.api_key = mask_api_key(c.api_key.as_deref());
                    c
                })
                .collect();
            (
                StatusCode::OK,
                Json(json!({ "success": true, "configs": masked_configs })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

pub async fn add_model_config(
    State(state): State<AppState>,
    Json(payload): Json<AddConfigRequest>,
) -> impl IntoResponse {
    let key_to_save = match payload.api_key.as_deref() {
        Some(k) if k.contains("...") || k.contains("****") => {
            if let Ok(configs) = state.db.list_model_configs().await {
                configs
                    .into_iter()
                    .find(|c| c.provider == payload.provider && c.model_id == payload.model_id)
                    .and_then(|c| c.api_key)
            } else {
                None
            }
        }
        other => other.map(|s| s.to_string()),
    };

    match state
        .db
        .add_model_config(
            &payload.name,
            &payload.provider,
            payload.base_url.as_deref(),
            key_to_save.as_deref(),
            &payload.model_id,
            payload.context_length.unwrap_or(4096),
            None,
        )
        .await
    {
        Ok(mut cfg) => {
            cfg.api_key = mask_api_key(cfg.api_key.as_deref());
            (
                StatusCode::CREATED,
                Json(json!({ "success": true, "config": cfg })),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

pub async fn activate_model_config(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.db.set_active_model(&id).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "success": true }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

pub async fn delete_model_config(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.db.delete_model_config(&id).await {
        Ok(()) => (StatusCode::OK, Json(json!({ "success": true }))),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

pub async fn test_connection(
    State(state): State<AppState>,
    Json(payload): Json<TestConnectionRequest>,
) -> impl IntoResponse {
    let resolved_key = match payload.api_key.as_deref() {
        Some(k) if !k.is_empty() && !k.contains("...") && !k.contains("****") => {
            Some(k.to_string())
        }
        _ => {
            if let Ok(configs) = state.db.list_model_configs().await {
                configs
                    .into_iter()
                    .find(|c| c.provider == payload.provider && c.model_id == payload.model_id)
                    .and_then(|c| c.api_key)
            } else {
                None
            }
        }
    };

    let res = state
        .router
        .test_provider(
            &payload.provider,
            payload.base_url.as_deref(),
            resolved_key.as_deref(),
            &payload.model_id,
        )
        .await;

    Json(json!({
        "success": res.success,
        "message": res.message,
        "latency_ms": res.latency_ms,
        "models": res.models,
    }))
}

#[derive(Deserialize)]
pub struct ConfigureRoleRequest {
    pub role: String, // "vision" or "translation" or "main"
    pub name: String,
    pub provider: String,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model_id: String,
}

pub async fn get_roles_handler(State(state): State<AppState>) -> impl IntoResponse {
    let main_cfg = state.db.get_active_model_config().await.unwrap_or(None);
    let vision_cfg = state.db.get_active_vision_model_config().await.unwrap_or(None);
    let trans_cfg = state.db.get_active_translation_model_config().await.unwrap_or(None);

    Json(json!({
        "success": true,
        "main": main_cfg,
        "vision": vision_cfg,
        "translation": trans_cfg,
    }))
}

pub async fn configure_role_handler(
    State(state): State<AppState>,
    Json(payload): Json<ConfigureRoleRequest>,
) -> impl IntoResponse {
    let flags = json!({ "role": payload.role }).to_string();
    let res = state
        .db
        .add_model_config(
            &payload.name,
            &payload.provider,
            payload.base_url.as_deref(),
            payload.api_key.as_deref(),
            &payload.model_id,
            4096,
            Some(&flags),
        )
        .await;

    match res {
        Ok(cfg) => {
            if payload.role == "vision" {
                let _ = state.db.set_active_vision_model(&cfg.id).await;
            } else if payload.role == "translation" {
                let _ = state.db.set_active_translation_model(&cfg.id).await;
            } else {
                let _ = state.db.set_active_model(&cfg.id).await;
            }
            (StatusCode::OK, Json(json!({ "success": true, "config": cfg })))
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}
