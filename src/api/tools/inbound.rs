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
    pub protocol: String,
    pub event_type: String,
    pub source_id: Option<String>,
    pub channel_id: Option<String>,
    pub session_id: Option<String>,
    pub mode: Option<String>,
    pub payload: Value,
}

fn extract_field<'a>(payload: &'a Value, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|k| payload.get(*k).and_then(|v| v.as_str()))
}

fn extract_reply_info(payload: &Value) -> String {
    let q = payload.get("reply_to").or_else(|| payload.get("quoted_message"));
    match q {
        Some(Value::String(s)) if !s.is_empty() => format!("\nReplying To: \"{s}\""),
        Some(Value::Object(obj)) => {
            let text = obj.get("text").and_then(|t| t.as_str()).unwrap_or("");
            let sender = obj.get("sender").and_then(|s| s.as_str());
            if text.is_empty() {
                String::new()
            } else if let Some(s) = sender {
                format!("\nReplying To: \"{text}\" (by {s})")
            } else {
                format!("\nReplying To: \"{text}\"")
            }
        }
        _ => String::new(),
    }
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
        db.find_session_for_tool(protocol, cid, sender_name).await.ok().flatten()
    } else if let Some(name) = sender_name {
        db.find_session_for_tool(protocol, name, None).await.ok().flatten()
    } else {
        None
    };

    if let Some(s) = existing {
        let title = if is_group && !s.title.is_empty() { s.title.clone() } else { canonical_title.to_string() };
        let _ = db.update_session_title_and_mode(&s.id, &title, mode).await;
        crate::db::ChatSession {
            id: s.id,
            title,
            mode: mode.to_string(),
            created_at: s.created_at,
            updated_at: chrono::Utc::now().to_rfc3339(),
        }
    } else {
        db.create_session(canonical_title, mode).await.unwrap_or_else(|_| crate::db::ChatSession {
            id: uuid::Uuid::new_v4().to_string(),
            title: canonical_title.to_string(),
            mode: mode.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            updated_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}

fn build_canonical_title(
    protocol: &str,
    is_group: bool,
    group_subject: Option<&str>,
    channel_id: Option<&str>,
    sender_display: &str,
) -> String {
    let proto = protocol.to_uppercase();
    if is_group {
        match (group_subject, channel_id) {
            (Some(sub), Some(cid)) => format!("[{proto}] {sub} ({cid})"),
            (Some(sub), None) => format!("[{proto}] {sub}"),
            (None, Some(cid)) => format!("[{proto}] Group ({cid})"),
            (None, None) => format!("[{proto}] Group Chat"),
        }
    } else if let Some(cid) = channel_id {
        format!("[{proto}] {sender_display} ({cid})")
    } else {
        format!("[{proto}] {sender_display}")
    }
}

async fn resolve_or_create_session(
    db: &crate::db::Database,
    protocol: &str,
    channel_id: Option<&str>,
    group_subject: Option<&str>,
    sender_display: &str,
    mode: &str,
    is_group: bool,
) -> String {
    let title = build_canonical_title(protocol, is_group, group_subject, channel_id, sender_display);
    let s = find_or_create_tool_session(db, protocol, channel_id, Some(sender_display), &title, mode, is_group).await;
    s.id
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

    let extracted_channel_id = payload.channel_id.as_deref().or_else(|| {
        extract_field(&payload.payload, &["channel_id", "chat_jid", "from", "sender_jid", "sender"])
    });

    let sender_display = extract_field(&payload.payload, &["sender_name", "sender"])
        .or(payload.source_id.as_deref())
        .unwrap_or("External Contact");

    let group_subject = extract_field(&payload.payload, &["group_name", "group_subject", "chat_name", "subject"]);
    let is_group = payload.payload.get("is_group").and_then(|v| v.as_bool()).unwrap_or_else(|| group_subject.is_some());

    let text = extract_field(&payload.payload, &["message", "text", "body"]).unwrap_or("");
    let reply_info = extract_reply_info(&payload.payload);

    let is_memory_ignored = extracted_channel_id
        .map(|cid| state.memory.is_channel_ignored(cid))
        .unwrap_or(false);

    let (should_trigger, effective_policy, reason) = if let Some(cid) = extracted_channel_id {
        let op_name = state.db.get_setting("operator_name").await.unwrap_or(None).unwrap_or_default();
        let ch_name = group_subject.or(Some(sender_display));
        state
            .db
            .evaluate_channel_trigger(
                cid,
                &payload.protocol,
                ch_name,
                is_group,
                text,
                &reply_info,
                &op_name,
                is_memory_ignored,
            )
            .await
            .unwrap_or((true, "all".into(), "Defaulted due to evaluation error".into()))
    } else {
        (true, "all".into(), "No channel identifier present".into())
    };

    if !should_trigger {
        info!(
            channel_id = ?extracted_channel_id,
            protocol = %payload.protocol,
            policy = %effective_policy,
            reason = %reason,
            "Inbound tool event ignored per granular channel trigger policy"
        );
        return (
            StatusCode::OK,
            Json(json!({
                "success": true,
                "ignored": true,
                "policy": effective_policy,
                "reason": reason
            })),
        );
    }

    let target_session_id = match payload.session_id.as_deref() {
        Some(sid) if state.db.get_session(sid).await.ok().flatten().is_some() => sid.to_string(),
        Some(stale_sid) => {
            tracing::warn!(stale_sid, channel = ?extracted_channel_id, "Stale session_id; resolving canonical session");
            resolve_or_create_session(&state.db, &payload.protocol, extracted_channel_id, group_subject, sender_display, target_mode, is_group).await
        }
        None => {
            resolve_or_create_session(&state.db, &payload.protocol, extracted_channel_id, group_subject, sender_display, target_mode, is_group).await
        }
    };

    let channel_desc = if is_group {
        match (group_subject, extracted_channel_id) {
            (Some(sub), Some(cid)) => format!("Group: {sub} ({cid})"),
            (Some(sub), None) => format!("Group: {sub}"),
            (None, Some(cid)) => format!("Group ({cid})"),
            (None, None) => "Group Chat".to_string(),
        }
    } else if let Some(cid) = extracted_channel_id {
        format!("Direct Chat ({cid})")
    } else {
        "Direct Chat".to_string()
    };

    let turn_prompt = match payload.event_type.as_str() {
        "new_message" | "message_received" => format!(
            "[INCOMING TOOL EVENT: {}]\nSender: {}\nChannel: {}{}\nMessage: \"{}\"",
            payload.protocol.to_uppercase(), sender_display, channel_desc, reply_info, text
        ),
        _ => {
            let pretty = serde_json::to_string_pretty(&payload.payload).unwrap_or_else(|_| "{}".to_string());
            format!("[INCOMING TOOL EVENT: {}]\nEvent: {}\nSource: {}\nDetails: {}", payload.protocol.to_uppercase(), payload.event_type, sender_display, pretty)
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

    let incoming_images = extract_field(&payload.payload, &["media_url", "image_url"]).map(|u| vec![u.to_string()]);

    let (turn_id, position, is_immediate) = state
        .turn_queue
        .submit_turn_with_images(&target_session_id, &turn_prompt, target_mode, &format!("tool_{}", payload.protocol), incoming_images, event_tx)
        .await;

    // Send proactive WebSocket notification to notify operator in UI and trigger session refresh
    let notif_msg = json!({
        "topic": "notification",
        "payload": {
            "id": format!("inbound-{}", turn_id),
            "title": format!("{} from {}", payload.protocol.to_uppercase(), sender_display),
            "message": if text.is_empty() { format!("Incoming {} event", payload.event_type) } else { text.to_string() },
            "urgency": "info",
            "sessionId": target_session_id,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }
    });
    let _ = state.ws_broadcast.send(notif_msg.to_string());

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
