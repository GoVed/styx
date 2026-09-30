use axum::extract::Query;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::docker::engines::{get_preset_templates, DeployModelRequest};

#[derive(Deserialize)]
pub struct QuantRecommendQuery {
    pub model_size: Option<f32>,
    pub vram_gb: Option<usize>,
    pub context: Option<usize>,
}

pub async fn list_presets() -> impl IntoResponse {
    Json(json!({
        "success": true,
        "presets": get_preset_templates()
    }))
}

pub async fn list_local_model_files() -> impl IntoResponse {
    let mut files = Vec::new();
    let mut searched_dirs = Vec::new();

    searched_dirs.push(std::path::PathBuf::from("/models"));
    if let Ok(dir) = std::env::var("HOST_LOCAL_MODELS_DIR") {
        searched_dirs.push(std::path::PathBuf::from(dir));
    }
    if let Ok(home) = std::env::var("HOST_HOME").or_else(|_| std::env::var("HOME")) {
        searched_dirs.push(std::path::PathBuf::from(format!("{}/localLLM/models", home)));
        searched_dirs.push(std::path::PathBuf::from(format!("{}/models", home)));
    }

    let mut seen_filenames = std::collections::HashSet::new();

    for dir in searched_dirs {
        if dir.exists()
            && let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file()
                        && let Some(ext) = path.extension()
                            && ext == "gguf" {
                                let filename = path
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .to_string();
                                if seen_filenames.insert(filename.clone()) {
                                    let meta = entry.metadata().ok();
                                    let size_bytes = meta.map(|m| m.len()).unwrap_or(0);
                                    let size_gb = ((size_bytes as f64)
                                        / (1024.0 * 1024.0 * 1024.0)
                                        * 10.0)
                                        .round()
                                        / 10.0;
                                    let is_mmproj = filename.to_lowercase().contains("mmproj");
                                    let is_draft = filename.to_lowercase().contains("mtp")
                                        || filename.to_lowercase().contains("draft");
                                    files.push(json!({
                                        "filename": filename,
                                        "path": format!("/models/{}", filename),
                                        "size_gb": size_gb,
                                        "is_mmproj": is_mmproj,
                                        "is_draft": is_draft,
                                    }));
                                }
                            }
                }
            }
    }

    files.sort_by(|a, b| {
        let a_str = a.get("filename").and_then(|f| f.as_str()).unwrap_or("");
        let b_str = b.get("filename").and_then(|f| f.as_str()).unwrap_or("");
        a_str.cmp(b_str)
    });

    Json(json!({ "success": true, "files": files }))
}

pub async fn preview_docker_command(Json(req): Json<DeployModelRequest>) -> impl IntoResponse {
    let preview = req.generate_docker_cli_preview();
    Json(json!({ "success": true, "command": preview }))
}

pub async fn get_quantization_recommendation(
    Query(query): Query<QuantRecommendQuery>,
) -> impl IntoResponse {
    let model_size = query.model_size.unwrap_or(27.0);
    let vram_gb = query.vram_gb.unwrap_or(24);
    let context = query.context.unwrap_or(65536);
    let rec = crate::docker::engines::recommend_quantization(model_size, vram_gb, context);
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "recommendation": rec
        })),
    )
}
