use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::State;
use axum::http::{header, HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tracing::{debug, warn};

use crate::hitl::ApprovalDecision;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/", get(ws_handler))
}

fn is_origin_allowed(origin: &str, headers: &HeaderMap, config: &crate::config::AppConfig) -> bool {
    let origin_clean = origin.trim().trim_end_matches('/').to_lowercase();

    // 1. If explicit allowed origins configured, check match
    if !config.allowed_origins.is_empty() {
        return config.allowed_origins.iter().any(|o| {
            let o_clean = o.trim().trim_end_matches('/').to_lowercase();
            origin_clean == o_clean
        });
    }

    // 2. Allow localhost and loopback interfaces
    if origin_clean.starts_with("http://localhost")
        || origin_clean.starts_with("https://localhost")
        || origin_clean.starts_with("http://127.0.0.1")
        || origin_clean.starts_with("https://127.0.0.1")
    {
        return true;
    }

    // 3. Allow matching Host header
    if let Some(host) = headers.get(header::HOST).and_then(|h| h.to_str().ok()) {
        let host_clean = host.trim().to_lowercase();
        if origin_clean.ends_with(&host_clean)
            || origin_clean.replace("https://", "").replace("http://", "") == host_clean
        {
            return true;
        }
    }

    false
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    headers: HeaderMap,
    uri: Uri,
) -> Response {
    // 1. Validate Origin header to prevent Cross-Site WebSocket Hijacking (CSWSH)
    if let Some(origin) = headers.get(header::ORIGIN).and_then(|h| h.to_str().ok())
        && !is_origin_allowed(origin, &headers, &state.config) {
            warn!("Rejected WebSocket connection from untrusted origin: {}", origin);
            return (
                StatusCode::FORBIDDEN,
                axum::Json(json!({
                    "success": false,
                    "error": "Forbidden: Untrusted WebSocket Origin"
                })),
            )
                .into_response();
        }

    // 2. Verify Authentication Token
    if !crate::api::auth::verify_token_from_parts(&state, &headers, &uri).await {
        warn!("Rejected unauthenticated WebSocket connection attempt");
        return (
            StatusCode::UNAUTHORIZED,
            axum::Json(json!({
                "success": false,
                "error": "Unauthorized: Valid Device Access Key required"
            })),
        )
            .into_response();
    }

    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut ws_rx = state.ws_broadcast.subscribe();

    // Send initial handshake with pending tickets and telemetry
    let pending_tickets = state.hitl.list_pending_tickets().await;
    let initial_msg = json!({
        "topic": "connected",
        "pending_tickets": pending_tickets,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    let _ = sender.send(Message::Text(initial_msg.to_string().into())).await;

    // Background task to pump broadcast events to the websocket client
    let mut forward_task = tokio::spawn(async move {
        while let Ok(msg_str) = ws_rx.recv().await {
            if let Err(e) = sender.send(Message::Text(msg_str.into())).await {
                debug!("WebSocket client disconnected: {:?}", e);
                break;
            }
        }
    });

    // Handle incoming messages from the frontend client
    let state_clone = state.clone();
    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    let text_str = text.to_string();
                    if let Ok(val) = serde_json::from_str::<Value>(&text_str) {
                        let action = val.get("action").and_then(|a| a.as_str()).unwrap_or("");
                        match action {
                            "ping" => {
                                let pong = json!({ "topic": "pong", "time": chrono::Utc::now().to_rfc3339() });
                                let _ = state_clone.ws_broadcast.send(pong.to_string());
                            }
                            "decide" => {
                                let ticket_id = val.get("ticket_id").and_then(|t| t.as_str()).unwrap_or("");
                                let decision_type = val.get("decision").and_then(|d| d.as_str()).unwrap_or("");
                                let reason = val.get("reason").and_then(|r| r.as_str()).map(|s| s.to_string());
                                let mod_args = val.get("modified_arguments").cloned();

                                let decision = match decision_type.to_uppercase().as_str() {
                                    "APPROVE" => ApprovalDecision::Approve,
                                    "REJECT" => ApprovalDecision::Reject { reason },
                                    "MODIFY" => {
                                        if let Some(args) = mod_args {
                                            ApprovalDecision::ModifyPayload { new_arguments: args }
                                        } else {
                                            continue;
                                        }
                                    }
                                    _ => continue,
                                };

                                let _ = state_clone.hitl.resolve_approval(ticket_id, decision).await;
                            }
                            "stream_logs" => {
                                let container_id = val.get("container_id").and_then(|c| c.as_str()).unwrap_or("").to_string();
                                if !container_id.is_empty() {
                                    let state_for_logs = state_clone.clone();
                                    tokio::spawn(async move {
                                        if let Ok(logs) = state_for_logs.docker.get_container_logs(&container_id, 50).await {
                                            for line in logs {
                                                let event = json!({
                                                    "topic": "docker_log",
                                                    "container_id": container_id,
                                                    "line": line
                                                });
                                                let _ = state_for_logs.ws_broadcast.send(event.to_string());
                                            }
                                        }
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    tokio::select! {
        _ = (&mut forward_task) => receive_task.abort(),
        _ = (&mut receive_task) => forward_task.abort(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;

    #[test]
    fn test_is_origin_allowed_defaults() {
        let mut config = AppConfig::from_env();
        config.allowed_origins = vec![];
        let mut headers = HeaderMap::new();
        headers.insert(header::HOST, "localhost:3000".parse().unwrap());

        // Localhost / Loopback origins allowed
        assert!(is_origin_allowed("http://localhost:3000", &headers, &config));
        assert!(is_origin_allowed("http://127.0.0.1:3000", &headers, &config));
        assert!(is_origin_allowed("https://localhost:3000", &headers, &config));

        // Matching host allowed
        assert!(is_origin_allowed("http://localhost:3000/", &headers, &config));

        // Untrusted foreign origins rejected
        assert!(!is_origin_allowed("https://malicious-site.com", &headers, &config));
        assert!(!is_origin_allowed("http://evil-hacker.org", &headers, &config));
    }

    #[test]
    fn test_is_origin_allowed_custom_whitelist() {
        let mut config = AppConfig::from_env();
        config.allowed_origins = vec!["https://styx.mycompany.com".to_string()];
        let headers = HeaderMap::new();

        // Configured origin allowed
        assert!(is_origin_allowed("https://styx.mycompany.com", &headers, &config));
        assert!(is_origin_allowed("https://styx.mycompany.com/", &headers, &config));

        // Non-whitelisted origins rejected
        assert!(!is_origin_allowed("https://other-domain.com", &headers, &config));
    }
}
