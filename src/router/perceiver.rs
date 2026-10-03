use serde_json::{json, Value};

pub async fn resolve_vision_endpoint(db: &crate::db::Database) -> (String, String, String) {
    if let Ok(endpoint) = std::env::var("SYNDAE_VISION_ENDPOINT") {
        let model = std::env::var("SYNDAE_VISION_MODEL").unwrap_or_else(|_| "moondream".to_string());
        return (endpoint, String::new(), model);
    }
    if let Ok(Some(cfg)) = db.get_active_vision_model_config().await {
        let base = cfg.base_url.unwrap_or_else(|| "http://localhost:11434/v1".to_string());
        let key = cfg.api_key.unwrap_or_default();
        return (base, key, cfg.model_id);
    }
    if let Ok(Some(active)) = db.get_active_model_config().await {
        let lower = active.model_id.to_lowercase();
        if lower.contains("qwen3.5-9b")
            || lower.contains("qwen2.5-vl")
            || lower.contains("qwen2-vl")
            || active.provider == "anthropic"
            || active.provider == "gemini"
            || active.provider == "openai"
        {
            let base = active.base_url.unwrap_or_else(|| "http://localhost:8080/v1".to_string());
            let key = active.api_key.unwrap_or_default();
            return (base, key, active.model_id);
        }
    }
    (
        "http://localhost:11434/v1".to_string(),
        String::new(),
        "moondream".to_string(),
    )
}

pub use super::vision::inspect_image_with_model;

pub async fn perceive_or_strip_images(
    messages: &[Value],
    vision_endpoint: Option<(&str, &str, &str)>,
) -> Vec<Value> {
    let default_endpoint;
    let endpoint = if let Some(ep) = vision_endpoint {
        ep
    } else {
        let base = std::env::var("SYNDAE_VISION_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:11434/v1".to_string());
        let key = std::env::var("SYNDAE_VISION_KEY").unwrap_or_default();
        let model = std::env::var("SYNDAE_VISION_MODEL")
            .unwrap_or_else(|_| "moondream".to_string());
        default_endpoint = (base, key, model);
        (&default_endpoint.0[..], &default_endpoint.1[..], &default_endpoint.2[..])
    };

    let mut out = Vec::with_capacity(messages.len());
    for msg in messages {
        let mut cloned = msg.clone();
        if let Some(content_array) = cloned.get("content").and_then(|c| c.as_array()) {
            let mut text_parts = Vec::new();
            for part in content_array {
                if let Some(p_type) = part.get("type").and_then(|t| t.as_str()) {
                    if p_type == "text" {
                        if let Some(txt) = part.get("text").and_then(|t| t.as_str()) {
                            text_parts.push(txt.to_string());
                        }
                    } else if p_type == "image_url" {
                        let mut replaced = false;
                        if let Some(data_uri) = part.get("image_url").and_then(|u| u.get("url")).and_then(|s| s.as_str()) {
                            if let Ok(analysis) = inspect_image_with_model(
                                endpoint.0,
                                endpoint.1,
                                endpoint.2,
                                data_uri,
                                "Describe what is shown in this image thoroughly: identify all visible text, people, objects, activities, scene setting, and key details.",
                            ).await {
                                if !analysis.trim().is_empty() {
                                    text_parts.push(format!("[Visual Observation from Vision Model: {}]", analysis.trim()));
                                    replaced = true;
                                }
                            }
                        }
                        if !replaced {
                            text_parts.push("[Image attached]".to_string());
                        }
                    }
                }
            }
            cloned["content"] = json!(text_parts.join("\n"));
        }
        out.push(cloned);
    }
    out
}
