use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::mpsc;
use tracing::info;

use crate::agent::AgentEvent;
use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct InboundToolEventRequest {
    pub protocol: String,           // e.g. "chat", "email", "system", "webhook"
    pub event_type: String,         // e.g. "new_message", "incoming_email", "alert"
    pub source_id: Option<String>,  // e.g. "Person X" or "+123456789"
    pub channel_id: Option<String>, // e.g. "channel_123" or "+123456789"
    pub session_id: Option<String>, // Optional: existing session ID
    pub mode: Option<String>,       // Optional: "chat" or "mission"
    pub payload: Value,
}

pub async fn find_or_create_tool_session(
    db: &crate::db::Database,
    protocol: &str,
    channel_id: Option<&str>,
    sender_name: Option<&str>,
    canonical_title: &str,
    mode: &str,
    is_group: bool,
) -> crate::db::ChatSession {
    let existing = if let Some(cid) = channel_id {
        db.find_session_for_tool(protocol, cid, sender_name)
            .await
            .ok()
            .flatten()
    } else if let Some(name) = sender_name {
        db.find_session_for_tool(protocol, name, None)
            .await
            .ok()
            .flatten()
    } else {
        None
    };

    if let Some(s) = existing {
        // If it's a group session and already has a title, maintain it so participants don't overwrite it
        let title_to_use = if is_group && !s.title.is_empty() {
            s.title.clone()
        } else {
            canonical_title.to_string()
        };
        let _ = db
            .update_session_title_and_mode(&s.id, &title_to_use, mode)
            .await;
        crate::db::ChatSession {
            id: s.id,
            title: title_to_use,
            mode: mode.to_string(),
            created_at: s.created_at,
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    } else {
        db.create_session(canonical_title, mode)
            .await
            .unwrap_or_else(|_| crate::db::ChatSession {
                id: uuid::Uuid::new_v4().to_string(),
                title: canonical_title.to_string(),
                mode: mode.to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: chrono::Utc::now().to_rfc3339(),
            })
    }
}

pub async fn handle_inbound_tool_event(
    State(state): State<AppState>,
    Json(payload): Json<InboundToolEventRequest>,
) -> impl IntoResponse {
    let default_mode = match payload.event_type.as_str() {
        "new_message" | "message_received" => "chat",
        _ => "mission",
    };
    let target_mode = payload.mode.as_deref().unwrap_or(default_mode);

    let extracted_channel_id = payload
        .channel_id
        .clone()
        .or_else(|| {
            payload
                .payload
                .get("chat_jid")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            payload
                .payload
                .get("from")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            payload
                .payload
                .get("sender_jid")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        })
        .or_else(|| {
            payload
                .payload
                .get("sender")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
        });

    let sender_display = payload
        .payload
        .get("sender_name")
        .and_then(|v| v.as_str())
        .or_else(|| {
            payload
                .payload
                .get("sender")
                .and_then(|v| v.as_str())
        })
        .or(payload.source_id.as_deref())
        .unwrap_or("External Contact");

    let group_subject = payload
        .payload
        .get("group_name")
        .and_then(|v| v.as_str())
        .or_else(|| payload.payload.get("subject").and_then(|v| v.as_str()));

    let is_group = payload
        .payload
        .get("is_group")
        .and_then(|v| v.as_bool())
        .unwrap_or_else(|| group_subject.is_some());

    // Check if the incoming channel or source is explicitly ignored in operator memory (user_profile.md)
    let is_ignored = extracted_channel_id
        .as_deref()
        .map(|cid| state.memory.is_channel_ignored(cid))
        .unwrap_or(false);

    if is_ignored {
        info!(
            channel_id = ?extracted_channel_id,
            protocol = %payload.protocol,
            "Ignoring incoming event as channel is marked as ignored in operator memory directives"
        );
        return (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "ignored": true,
                "reason": "Channel or contact marked as ignored in memory directives"
            })),
        );
    }

    let target_session_id = if let Some(ref sid) = payload.session_id {
        sid.clone()
    } else {
        let canonical_title = if is_group {
            if let Some(sub) = group_subject {
                format!("[{}] {}", payload.protocol.to_uppercase(), sub)
            } else if let Some(ref cid) = extracted_channel_id {
                format!("[{}] Group ({})", payload.protocol.to_uppercase(), cid)
            } else {
                format!("[{}] Group Chat", payload.protocol.to_uppercase())
            }
        } else if let Some(ref cid) = extracted_channel_id {
            format!(
                "[{}] {} ({})",
                payload.protocol.to_uppercase(),
                sender_display,
                cid
            )
        } else {
            format!("[{}] {}", payload.protocol.to_uppercase(), sender_display)
        };

        let session = find_or_create_tool_session(
            &state.db,
            &payload.protocol,
            extracted_channel_id.as_deref(),
            Some(sender_display),
            &canonical_title,
            target_mode,
            is_group,
        )
        .await;
        session.id
    };

    let turn_prompt = match payload.event_type.as_str() {
        "new_message" | "message_received" => {
            let text = payload
                .payload
                .get("message")
                .or_else(|| payload.payload.get("text"))
                .or_else(|| payload.payload.get("body"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let channel_desc = if is_group {
                if let Some(sub) = group_subject {
                    format!("Group: {}", sub)
                } else {
                    "Group Chat".to_string()
                }
            } else if let Some(ref cid) = extracted_channel_id {
                format!("Direct Chat ({})", cid)
            } else {
                "Direct Chat".to_string()
            };

            let reply_info = payload
                .payload
                .get("reply_to")
                .or_else(|| payload.payload.get("quoted_message"))
                .and_then(|q| {
                    if let Some(s) = q.as_str() {
                        if !s.is_empty() {
                            Some(format!("\nReplying To: \"{}\"", s))
                        } else {
                            None
                        }
                    } else if let Some(obj) = q.as_object() {
                        let q_text = obj.get("text").and_then(|t| t.as_str()).unwrap_or("");
                        let q_sender = obj.get("sender").and_then(|s| s.as_str());
                        if !q_text.is_empty() {
                            if let Some(s) = q_sender {
                                Some(format!("\nReplying To: \"{}\" (by {})", q_text, s))
                            } else {
                                Some(format!("\nReplying To: \"{}\"", q_text))
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            format!(
                "[INCOMING TOOL EVENT: {}]\nSender: {}\nChannel: {}{}\nMessage: \"{}\"",
                payload.protocol.to_uppercase(),
                sender_display,
                channel_desc,
                reply_info,
                text
            )
        }
        _ => {
            let pretty_payload =
                serde_json::to_string_pretty(&payload.payload).unwrap_or_else(|_| "{}".to_string());
            format!(
                "[INCOMING TOOL EVENT: {}]\nEvent: {}\nSource: {}\nDetails: {}",
                payload.protocol.to_uppercase(),
                payload.event_type,
                sender_display,
                pretty_payload
            )
        }
    };

    let (event_tx, mut event_rx) = mpsc::channel::<AgentEvent>(100);
    let ws_broadcast = state.ws_broadcast.clone();
    let session_id_broadcast = target_session_id.clone();

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

    let incoming_images = payload
        .payload
        .get("media_url")
        .or_else(|| payload.payload.get("image_url"))
        .and_then(|v| v.as_str())
        .map(|u| vec![u.to_string()]);

    let (turn_id, position, is_immediate) = state
        .turn_queue
        .submit_turn_with_images(
            &target_session_id,
            &turn_prompt,
            target_mode,
            &format!("tool_{}", payload.protocol),
            incoming_images,
            event_tx,
        )
        .await;

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "session_id": target_session_id,
            "turn_id": turn_id,
            "status": if is_immediate { "running" } else { "queued" },
            "queue_position": position,
            "protocol": payload.protocol,
            "event_type": payload.event_type,
            "message": if is_immediate {
                "Inbound tool event received and dispatched to agent execution"
            } else {
                "Inbound tool event received and queued for GPU inference slot"
            }
        })),
    )
}
