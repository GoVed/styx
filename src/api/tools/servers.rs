use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::mcp::policy::PolicyTier;
use crate::mcp::transport::McpTransport;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct AddServerRequest {
    pub name: String,
    pub transport_type: String, // "stdio", "unix_socket", "http_sse", "http"
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
    pub env: Option<HashMap<String, String>>,
    pub socket_path: Option<String>,
    pub url: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdatePolicyRequest {
    pub tool_name: String,
    pub policy: String, // "AUTONOMOUS", "REQUIRE_APPROVAL", "BLOCKED"
    pub risk_level: Option<String>,
}

#[derive(Deserialize)]
pub struct CallToolRequest {
    pub tool_name: String,
    pub arguments: Value,
}

pub async fn list_servers(State(state): State<AppState>) -> impl IntoResponse {
    match state.db.list_mcp_servers().await {
        Ok(servers) => (
            StatusCode::OK,
            Json(json!({ "success": true, "servers": servers })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

pub async fn add_server(
    State(state): State<AppState>,
    Json(payload): Json<AddServerRequest>,
) -> impl IntoResponse {
    let transport_res = match payload.transport_type.as_str() {
        "stdio" => {
            let cmd = payload.command.clone().unwrap_or_default();
            let args = payload.args.clone().unwrap_or_default();
            let env_vec: Vec<(String, String)> = payload
                .env
                .clone()
                .unwrap_or_default()
                .into_iter()
                .collect();
            McpTransport::connect_stdio(&cmd, &args, &env_vec).await
        }
        "unix_socket" => {
            let path = payload.socket_path.clone().unwrap_or_default();
            McpTransport::connect_unix(&path).await
        }
        "http_sse" | "http" => {
            let url = payload.url.clone().unwrap_or_default();
            Ok(McpTransport::connect_http(&url))
        }
        _ => Err(anyhow::anyhow!(
            "Unsupported transport type: {}",
            payload.transport_type
        )),
    };

    let transport = match transport_res {
        Ok(t) => t,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(
                    json!({ "success": false, "error": format!("Failed to connect to MCP server: {}", e) }),
                ),
            );
        }
    };

    let env_json = payload
        .env
        .as_ref()
        .map(|e| serde_json::to_string(e).unwrap_or_default());
    let args_json = payload
        .args
        .as_ref()
        .map(|a| serde_json::to_string(a).unwrap_or_default());

    let record = match state
        .db
        .add_mcp_server(
            &payload.name,
            &payload.transport_type,
            payload.command.as_deref(),
            args_json.as_deref(),
            env_json.as_deref(),
            payload.socket_path.as_deref(),
            payload.url.as_deref(),
        )
        .await
    {
        Ok(r) => r,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "success": false, "error": format!("Database error: {}", e) })),
            );
        }
    };

    // Load persisted tool policies
    let saved_policies = state.db.list_tool_policies().await.unwrap_or_default();
    let mut policy_map = HashMap::new();
    for p in saved_policies {
        let tier = match p.policy.as_str() {
            "AUTONOMOUS" => PolicyTier::Autonomous,
            "BLOCKED" => PolicyTier::Blocked,
            _ => PolicyTier::RequireApproval,
        };
        policy_map.insert(p.tool_name, tier);
    }

    match state
        .mcp
        .register_server(&record.id, transport, &policy_map)
        .await
    {
        Ok(tools) => {
            // Automatically fetch and install tool instructions/skills if an HTTP endpoint is provided
            if let Some(ref url) = payload.url {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_millis(1500))
                    .build()
                    .unwrap_or_default();
                let inst_url = url.replace("/mcp", "/instructions");
                if let Ok(resp) = client.get(&inst_url).send().await
                    && let Ok(val) = resp.json::<Value>().await
                        && let Some(inst) = val.get("instructions").and_then(|i| i.as_str()) {
                            let clean_name = payload.name.to_lowercase().replace('@', "").replace('/', "_");
                            let rel_path = format!("skills/{}.md", clean_name);
                            let _ = state.memory.write_file(&rel_path, inst, None).await;
                        }
            }

            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "server": record,
                    "discovered_tools": tools
                })),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(
                json!({ "success": false, "error": format!("Handshake failed with MCP server: {}", e) }),
            ),
        ),
    }
}

pub async fn remove_server(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    state.mcp.unregister_server(&id).await;
    match state.db.delete_mcp_server(&id).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "success": true, "message": "Server removed" })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

pub async fn list_tools(State(state): State<AppState>) -> impl IntoResponse {
    let tools = state.mcp.list_tools().await;
    let policies = state.db.list_tool_policies().await.unwrap_or_default();

    let policy_map: HashMap<String, String> = policies
        .into_iter()
        .map(|p| (p.tool_name, p.policy))
        .collect();

    let mut enriched = Vec::new();
    for mut t in tools {
        if let Some(p_str) = policy_map.get(&t.name) {
            t.policy = PolicyTier::from_str_lenient(p_str);
        }
        enriched.push(t);
    }

    Json(json!({ "success": true, "tools": enriched }))
}

pub async fn update_policy(
    State(state): State<AppState>,
    Json(payload): Json<UpdatePolicyRequest>,
) -> impl IntoResponse {
    let tier = PolicyTier::from_str_lenient(&payload.policy);
    let _ = state.mcp.update_tool_policy(&payload.tool_name, tier).await;
    match state
        .db
        .set_tool_policy(
            &payload.tool_name,
            &payload.policy,
            payload.risk_level.as_deref(),
        )
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({ "success": true, "tool": payload.tool_name, "policy": payload.policy })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}

pub async fn test_call_tool(
    State(state): State<AppState>,
    Json(payload): Json<CallToolRequest>,
) -> impl IntoResponse {
    let is_builtin = state
        .mcp
        .get_tool(&payload.tool_name)
        .await
        .map(|t| t.server_id.is_none())
        .unwrap_or(false);

    let res = if is_builtin {
        let executor = crate::agent::AgentToolExecutor {
            db: &state.db,
            memory: &state.memory,
            mcp: &state.mcp,
            telemetry: &state.telemetry,
        };
        Ok(executor.execute_tool(&payload.tool_name, payload.arguments).await)
    } else {
        state
            .mcp
            .call_external_tool(&payload.tool_name, payload.arguments)
            .await
    };

    match res {
        Ok(output) => (
            StatusCode::OK,
            Json(json!({
                "success": output.success,
                "tool_name": output.tool_name,
                "content": output.content,
                "is_error": output.is_error
            })),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": e.to_string() })),
        ),
    }
}
