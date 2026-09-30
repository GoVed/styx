use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::json;

use crate::db::channels::ChannelDefaultsRecord;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct ListChannelsQuery {
    pub protocol: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateChannelPolicyRequest {
    pub channel_id: String,
    pub protocol: Option<String>,
    pub channel_name: Option<String>,
    pub is_group: Option<bool>,
    pub policy: String, // "all", "mentions", "muted", "manual"
    pub mention_keywords: Option<String>,
}

pub async fn list_channels(
    State(state): State<AppState>,
    Query(query): Query<ListChannelsQuery>,
) -> impl IntoResponse {
    match state.db.list_channel_policies(query.protocol.as_deref()).await {
        Ok(channels) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "channels": channels
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": e.to_string()
            })),
        ),
    }
}

pub async fn update_channel_policy(
    State(state): State<AppState>,
    Json(payload): Json<UpdateChannelPolicyRequest>,
) -> impl IntoResponse {
    let existing = state.db.get_channel_policy(&payload.channel_id).await.unwrap_or(None);
    let protocol = payload.protocol.as_deref().unwrap_or_else(|| {
        existing.as_ref().map(|e| e.protocol.as_str()).unwrap_or("whatsapp")
    });
    let channel_name = payload.channel_name.as_deref().or_else(|| {
        existing.as_ref().and_then(|e| e.channel_name.as_deref())
    });
    let is_group = payload.is_group.unwrap_or_else(|| {
        existing.as_ref().map(|e| e.is_group).unwrap_or_else(|| payload.channel_id.ends_with("@g.us"))
    });

    match state
        .db
        .set_channel_policy(
            &payload.channel_id,
            protocol,
            channel_name,
            is_group,
            &payload.policy,
            payload.mention_keywords.as_deref(),
        )
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message": "Channel trigger policy updated successfully",
                "channel_id": payload.channel_id,
                "policy": payload.policy
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": e.to_string()
            })),
        ),
    }
}

pub async fn get_channel_defaults(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.get_channel_defaults().await {
        Ok(defaults) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "defaults": defaults
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": e.to_string()
            })),
        ),
    }
}

pub async fn update_channel_defaults(
    State(state): State<AppState>,
    Json(payload): Json<ChannelDefaultsRecord>,
) -> impl IntoResponse {
    match state.db.set_channel_defaults(&payload).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "message": "Channel trigger defaults updated successfully",
                "defaults": payload
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": e.to_string()
            })),
        ),
    }
}
