use axum::{
    extract::State,
    http::{HeaderMap, StatusCode, Uri},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::mpsc;
use tracing::info;

use crate::agent::AgentEvent;
use crate::api::auth::middleware::verify_token_from_parts;
use crate::api::auth::rate_limit::{
    check_rate_limit, get_client_ip, record_failed_attempt, record_successful_attempt,
};
use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthStatusResponse {
    pub initialized: bool,
    pub onboarded: bool,
    pub operator_name: Option<String>,
    pub requires_auth: bool,
}

#[derive(Debug, Deserialize)]
pub struct AuthSetupPayload {
    pub access_key: String,
    pub operator_name: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AuthLoginPayload {
    pub access_key: String,
}

/// Query current single-user initialization and authentication status
pub async fn auth_status_handler(State(state): State<AppState>) -> Json<AuthStatusResponse> {
    let initialized = state.db.is_auth_initialized().await.unwrap_or(false);
    let onboarded = state.db.is_onboarded().await.unwrap_or(false);
    let operator_name = state.db.get_setting("operator_name").await.unwrap_or(None);

    Json(AuthStatusResponse {
        initialized,
        onboarded,
        operator_name,
        requires_auth: initialized,
    })
}

/// First-boot setup: configure device access key and operator identity
pub async fn auth_setup_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AuthSetupPayload>,
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

    let key_trimmed = payload.access_key.trim();
    if key_trimmed.len() < 8 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Access key must be at least 8 characters long for security" })),
        )
            .into_response();
    }

    if state.db.is_auth_initialized().await.unwrap_or(false) {
        record_failed_attempt(&client_ip).await;
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Device is already initialized. Please authenticate." })),
        )
            .into_response();
    }

    let hashed = crate::db::Database::hash_access_key(key_trimmed);
    if let Err(e) = state.db.set_setting("auth_access_key_hash", &hashed).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": format!("Failed to save access key: {}", e) })),
        )
            .into_response();
    }

    record_successful_attempt(&client_ip).await;

    let name = payload.operator_name.as_deref().unwrap_or("Operator");
    let _ = state.db.set_setting("operator_name", name).await;

    // Update initial user profile template in memory
    let role = payload.role.as_deref().unwrap_or("Personal User");
    let profile_content = format!(
        "# User Profile & Preferences\n\n\
         ## Identity\n\
         - **Name:** {}\n\
         - **Focus / Interests:** {}\n\
         - **System:** Styx Personal Assistant\n\
         - **Device:** Local Machine (100% Private)\n\n\
         ## How I Like to Communicate\n\
         - **Tone:** Friendly, conversational, clear, and direct. Avoid unnecessary developer jargon.\n\
         - **Format:** Interaction-based choices (<options>) with freedom to type custom answers.\n\
         - **Memory & Adaptation:** Continuously observe what I ask for and update this profile with my habits, routines, and preferred topics so you get better over time.\n",
        name, role
    );
    let _ = state
        .memory
        .write_file("core/user_profile.md", &profile_content, None)
        .await;

    info!(
        "Styx device initialized with access key and operator: {}",
        name
    );

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "token": key_trimmed,
            "operator_name": name,
            "onboarded": false
        })),
    )
        .into_response()
}

/// Verify access key credentials for single-user device unlock
pub async fn auth_login_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<AuthLoginPayload>,
) -> Response {
    let client_ip = get_client_ip(&headers);
    if let Err(secs) = check_rate_limit(&client_ip).await {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "success": false,
                "error": format!("Too many failed login attempts. Locked out for {} seconds.", secs)
            })),
        )
            .into_response();
    }

    match state.db.verify_access_key(&payload.access_key).await {
        Ok(true) => {
            record_successful_attempt(&client_ip).await;
            let onboarded = state.db.is_onboarded().await.unwrap_or(false);
            let operator_name = state.db.get_setting("operator_name").await.unwrap_or(None);
            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "token": payload.access_key.trim(),
                    "onboarded": onboarded,
                    "operator_name": operator_name
                })),
            )
                .into_response()
        }
        Ok(false) => {
            record_failed_attempt(&client_ip).await;
            (
                StatusCode::UNAUTHORIZED,
                Json(json!({ "success": false, "error": "Invalid Device Access Key" })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": format!("Auth error: {}", e) })),
        )
            .into_response(),
    }
}

/// Check if bearer token / access key is valid
pub async fn auth_verify_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    if verify_token_from_parts(&state, &headers, &uri).await {
        let onboarded = state.db.is_onboarded().await.unwrap_or(false);
        let operator_name = state.db.get_setting("operator_name").await.unwrap_or(None);
        (
            StatusCode::OK,
            Json(json!({
                "valid": true,
                "onboarded": onboarded,
                "operator_name": operator_name
            })),
        )
            .into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "valid": false, "error": "Invalid or expired access token" })),
        )
            .into_response()
    }
}

/// Create onboarding session and kick off introductory interview turn
pub async fn start_onboarding_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    if !verify_token_from_parts(&state, &headers, &uri).await {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "success": false, "error": "Unauthorized" })),
        )
            .into_response();
    }

    // Create onboarding session
    let session = match state
        .db
        .create_session("[ONBOARDING] Welcome to Styx", "chat")
        .await
    {
        Ok(s) => s,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": format!("Failed to create session: {}", e) })),
            )
                .into_response();
        }
    };

    let operator_hint = state
        .db
        .get_setting("operator_name")
        .await
        .ok()
        .flatten()
        .unwrap_or_else(|| "Operator".to_string());

    let name_greeting = if operator_hint == "Operator" || operator_hint.is_empty() {
        "there".to_string()
    } else {
        operator_hint
    };

    let onboarding_starter_prompt = format!(
        "Hello Styx! I just set up my personal assistant. Please start our friendly welcome and onboarding conversation for {}.\n\n\
         GUIDANCE FOR GENERAL PUBLIC INTERACTION:\n\
         You are talking to an everyday person, not a developer. Be warm, natural, and friendly. Never use technical jargon.\n\
         Keep your response short (1-2 sentences) and interactive:\n\
         1. Welcome me warmly to my personal assistant.\n\
         2. Ask: 'What would you like to use me for most?'\n\
         3. Provide 4 everyday options plus 'Other' using <options> tags:\n\
            <options>\n\
            <option>Daily Productivity & Organizing (tasks, notes, reminders)</option>\n\
            <option>Writing, Brainstorming & Creative Ideas</option>\n\
            <option>Work, Business & Professional Projects</option>\n\
            <option>Learning, Studying & Exploring New Topics</option>\n\
            <option other=\"true\">Other (tell me in your own words)...</option>\n\
            </options>\n\
         4. Over our first few messages, ask about my preferred communication style, learn about my daily habits, and save my preferences to `core/user_profile.md` so you adapt to how I behave over time.\n\
         5. Once my profile and preferences are recorded, call the `complete_onboarding` tool so the system marks enrollment as completed.",
        name_greeting
    );

    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(100);
    let ws_broadcast = state.ws_broadcast.clone();
    let session_id_broadcast = session.id.clone();

    // Spawn event relay to WebSocket broadcast
    tokio::spawn(async move {
        while let Some(evt) = event_rx.recv().await {
            let msg = json!({
                "topic": "chat_event",
                "session_id": session_id_broadcast,
                "event": evt
            });
            let _ = ws_broadcast.send(msg.to_string());
        }
    });

    let (turn_id, position, is_immediate) = state
        .turn_queue
        .submit_turn(
            &session.id,
            &onboarding_starter_prompt,
            "chat",
            "onboarding",
            event_tx,
        )
        .await;

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "session_id": session.id,
            "turn_id": turn_id,
            "position": position,
            "is_immediate": is_immediate,
            "message": "Onboarding session created and initial turn submitted to TurnQueue"
        })),
    )
        .into_response()
}

/// Mark onboarding as completed
pub async fn complete_onboarding_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    if !verify_token_from_parts(&state, &headers, &uri).await {
        return (
            StatusCode::UNAUTHORIZED,
            Json(json!({ "success": false, "error": "Unauthorized" })),
        )
            .into_response();
    }

    let _ = state.db.set_setting("onboarding_completed", "true").await;

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "message": "Onboarding completed successfully"
        })),
    )
        .into_response()
}
