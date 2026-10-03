use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::state::AppState;

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
pub struct ReasoningSettingResponse {
    pub success: bool,
    pub reasoning_effort: String,
    pub available_levels: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetReasoningRequest {
    pub reasoning_effort: String,
}

pub async fn get_reasoning_setting(State(state): State<AppState>) -> impl IntoResponse {
    let level = state
        .db
        .get_setting("reasoning_effort")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "high".to_string());

    Json(json!({
        "success": true,
        "reasoning_effort": level,
        "available_levels": ["high", "medium", "low", "off"]
    }))
}

pub async fn set_reasoning_setting(
    State(state): State<AppState>,
    Json(payload): Json<SetReasoningRequest>,
) -> impl IntoResponse {
    let level = payload.reasoning_effort.to_lowercase().trim().to_string();
    if !["high", "medium", "low", "off"].contains(&level.as_str()) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Invalid reasoning effort. Must be one of: high, medium, low, off"
            })),
        );
    }

    if let Err(e) = state.db.set_setting("reasoning_effort", &level).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        );
    }

    if let Ok(Some(active)) = state.db.get_active_model_config().await {
        let mut flags: serde_json::Map<String, serde_json::Value> = active
            .extra_flags_json
            .as_deref()
            .and_then(|f| serde_json::from_str(f).ok())
            .unwrap_or_default();
        flags.insert("reasoning_effort".to_string(), json!(level));
        let flags_str = serde_json::to_string(&flags).unwrap_or_default();
        let _ = state.db.update_model_config_flags(&active.id, Some(&flags_str)).await;
    }

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "reasoning_effort": level,
            "message": format!("Reasoning effort updated to {}", level)
        })),
    )
}

#[cfg(test)]
mod tests {
    use crate::db::Database;

    #[tokio::test]
    async fn test_reasoning_setting_default_and_mutation() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_reasoning_{}.db", uuid::Uuid::new_v4()));
        let db = Database::init(&db_path).expect("Failed to create test db");

        // 1. Initial state should default to "high"
        let initial = db.get_setting("reasoning_effort").await.unwrap();
        assert_eq!(initial, None);
        let default_val = initial.unwrap_or_else(|| "high".to_string());
        assert_eq!(default_val, "high");

        // 2. Set to medium
        db.set_setting("reasoning_effort", "medium").await.unwrap();
        let updated = db.get_setting("reasoning_effort").await.unwrap();
        assert_eq!(updated.as_deref(), Some("medium"));

        // 3. Set to off
        db.set_setting("reasoning_effort", "off").await.unwrap();
        let off_val = db.get_setting("reasoning_effort").await.unwrap();
        assert_eq!(off_val.as_deref(), Some("off"));

        let _ = std::fs::remove_file(db_path);
    }
}
