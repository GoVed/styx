use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::state::AppState;

pub async fn activate_container_as_model(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match state.docker.inspect_container(&id).await {
        Ok(info) => {
            let raw_name = info.name.unwrap_or_default();
            let clean_name = raw_name.trim_start_matches('/').to_string();
            let image = info
                .config
                .as_ref()
                .and_then(|c| c.image.clone())
                .unwrap_or_default();
            let cmd = info
                .config
                .as_ref()
                .and_then(|c| c.cmd.clone())
                .unwrap_or_default();

            let (provider, default_port) = if image.contains("vllm") {
                ("docker_vllm", 8000)
            } else if image.contains("llama.cpp") {
                ("docker_llamacpp", 8080)
            } else if image.contains("ollama") {
                ("docker_ollama", 11434)
            } else {
                ("docker_custom", 8000)
            };

            let mut port = default_port;
            if let Some(net) = &info.network_settings
                && let Some(ports) = &net.ports {
                    for v in ports.values() {
                        if let Some(bindings) = v
                            && let Some(first) = bindings.first()
                                && let Some(hp) = &first.host_port
                                    && let Ok(parsed) = hp.parse::<u16>() {
                                        port = parsed;
                                        break;
                                    }
                    }
                }

            let mut model_id = clean_name.clone();
            let mut context_length: i64 = 32768;

            if provider == "docker_vllm" {
                if let Some(pos) = cmd.iter().position(|a| a == "serve") {
                    if let Some(val) = cmd.get(pos + 1)
                        && !val.starts_with('-') {
                            model_id = val.clone();
                        }
                } else if let Some(first) = cmd.first()
                    && !first.starts_with('-') && first != "vllm" {
                        model_id = first.clone();
                    }
            } else if provider == "docker_llamacpp"
                && let Some(pos) = cmd.iter().position(|a| a == "-hf" || a == "-m")
                    && let Some(val) = cmd.get(pos + 1) {
                        model_id = val.clone();
                    }

            for (idx, arg) in cmd.iter().enumerate() {
                if (arg == "--max-model-len" || arg == "-c" || arg == "--ctx-size")
                    && let Some(val) = cmd.get(idx + 1)
                        && let Ok(ctx) = val.parse::<i64>() {
                            context_length = ctx;
                        }
            }

            let probe_url = if std::path::Path::new("/.dockerenv").exists() {
                format!("http://host.docker.internal:{}/v1/models", port)
            } else {
                format!("http://localhost:{}/v1/models", port)
            };

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_millis(1500))
                .build()
                .unwrap_or_default();

            if let Ok(resp) = client.get(&probe_url).send().await
                && resp.status().is_success()
                    && let Ok(body) = resp.json::<serde_json::Value>().await
                        && let Some(data) = body.get("data").and_then(|d| d.as_array())
                            && let Some(first_model) = data.first() {
                                if let Some(id_str) = first_model.get("id").and_then(|i| i.as_str()) {
                                    model_id = id_str.to_string();
                                }
                                if let Some(len) =
                                    first_model.get("max_model_len").and_then(|l| l.as_i64())
                                {
                                    context_length = len;
                                }
                            }

            let base_url = format!("http://localhost:{}/v1", port);
            let display_name = if model_id != clean_name {
                format!(
                    "Local {} ({})",
                    if provider == "docker_vllm" {
                        "vLLM"
                    } else if provider == "docker_llamacpp" {
                        "llama.cpp"
                    } else {
                        "Docker"
                    },
                    model_id
                )
            } else {
                format!(
                    "{} ({})",
                    clean_name,
                    if provider == "docker_vllm" {
                        "vLLM"
                    } else if provider == "docker_llamacpp" {
                        "llama.cpp"
                    } else {
                        "Docker"
                    }
                )
            };

            let existing_configs = state.db.list_model_configs().await.unwrap_or_default();
            let existing = existing_configs
                .into_iter()
                .find(|c| c.base_url.as_deref() == Some(&base_url) || c.model_id == model_id);

            let target_id = if let Some(found) = existing {
                let _ = state
                    .db
                    .update_model_config(&found.id, &display_name, &model_id, context_length)
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
                        context_length,
                        None,
                    )
                    .await
                {
                    Ok(rec) => rec.id,
                    Err(e) => {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({ "success": false, "error": e.to_string() })),
                        )
                    }
                }
            };

            match state.db.set_active_model(&target_id).await {
                Ok(()) => (
                    StatusCode::OK,
                    Json(json!({
                        "success": true,
                        "config_id": target_id,
                        "model_id": model_id,
                        "base_url": base_url
                    })),
                ),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "success": false, "error": e.to_string() })),
                ),
            }
        }
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "error": format!("Failed to inspect container: {}", e)
            })),
        ),
    }
}
