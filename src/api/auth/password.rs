use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use tracing::info;

use crate::api::auth::rate_limit::{
    check_rate_limit, get_client_ip, record_failed_attempt, record_successful_attempt,
};
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ChangePasswordPayload {
    pub current_password: String,
    pub new_password: String,
}

/// Change the device master access key / password from Settings
pub async fn change_password_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChangePasswordPayload>,
) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(secs) = check_rate_limit(&client_ip).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "success": false,
                "error": format!("Too many attempts. Rate limited for {} seconds.", secs)
            })),
        )
            .into_response();
    }

    let current = payload.current_password.trim();
    let new_key = payload.new_password.trim();

    if current.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "Current password is required"
            })),
        )
            .into_response();
    }

    // Verify current master access key
    match state.db.verify_access_key(current).await {
        Ok(true) => {
            // Authorized to change password
        }
        _ => {
            record_failed_attempt(&client_ip).await;
            return (
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "error": "Current password is incorrect"
                })),
            )
                .into_response();
        }
    }

    if new_key.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "New password must be at least 8 characters long"
            })),
        )
            .into_response();
    }

    if new_key == current {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "New password must be different from current password"
            })),
        )
            .into_response();
    }

    let hashed = crate::db::Database::hash_access_key(new_key);
    if let Err(e) = state.db.set_setting("auth_access_key_hash", &hashed).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": format!("Failed to update password: {}", e)
            })),
        )
            .into_response();
    }

    record_successful_attempt(&client_ip).await;
    info!("Device master password changed successfully");

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "message": "Password changed successfully",
            "token": new_key
        })),
    )
        .into_response()
}
