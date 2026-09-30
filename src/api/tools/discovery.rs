use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::mcp::transport::McpTransport;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct InstallToolRequest {
    pub path: Option<String>,
    pub name: String,
    pub transport_type: Option<String>,
    pub url: Option<String>,
    pub auto_start_container: Option<bool>,
}

pub async fn discover_local_tools(State(state): State<AppState>) -> impl IntoResponse {
    let mut discovered = Vec::new();
    let mut candidates = vec![
        "/tools".to_string(),
        "../styx_tools".to_string(),
        "./styx_tools".to_string(),
        "/app/styx_tools".to_string(),
    ];
    if let Ok(dir) = std::env::var("STYX_TOOLS_DIR") {
        candidates.insert(0, dir);
    }
    if let Ok(home) = std::env::var("HOST_HOME").or_else(|_| std::env::var("HOME")) {
        candidates.push(format!("{}/Projects/styx_tools", home));
        candidates.push(format!("{}/styx_tools", home));
    }

    let installed_servers = state.db.list_mcp_servers().await.unwrap_or_default();
    let mut checked_dirs = std::collections::HashSet::new();

    for base in &candidates {
        let base_path = std::path::Path::new(base);
        if base_path.is_dir()
            && let Ok(entries) = std::fs::read_dir(base_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let dir_name = path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        if checked_dirs.contains(&dir_name) {
                            continue;
                        }
                        checked_dirs.insert(dir_name.clone());

                        let manifest_path = path.join("manifest.json");
                        let pkg_json_path = path.join("package.json");
                        let compose_path = path.join("docker-compose.yml");

                        if manifest_path.exists() || pkg_json_path.exists() || compose_path.exists()
                        {
                            let mut name = dir_name.clone();
                            let mut display_name = dir_name.clone();
                            let mut desc = format!("Local Styx Tool Module ({})", dir_name);
                            let mut version = "1.0.0".to_string();
                            let mut has_docker = compose_path.exists();

                            // 1. Prioritize manifest.json
                            if let Ok(content) = std::fs::read_to_string(&manifest_path) {
                                if let Ok(val) = serde_json::from_str::<Value>(&content) {
                                    if let Some(n) = val.get("name").and_then(|v| v.as_str()) {
                                        name = n.to_string();
                                    }
                                    if let Some(dn) =
                                        val.get("display_name").and_then(|v| v.as_str())
                                    {
                                        display_name = dn.to_string();
                                    }
                                    if let Some(d) = val.get("description").and_then(|v| v.as_str())
                                    {
                                        desc = d.to_string();
                                    }
                                    if let Some(v) = val.get("version").and_then(|v| v.as_str()) {
                                        version = v.to_string();
                                    }
                                    if let Some(hd) = val.get("has_docker").and_then(|v| v.as_bool())
                                    {
                                        has_docker = hd;
                                    }
                                }
                            } else if let Ok(content) = std::fs::read_to_string(&pkg_json_path)
                                && let Ok(val) = serde_json::from_str::<Value>(&content) {
                                    if let Some(n) = val.get("name").and_then(|v| v.as_str()) {
                                        name = n.replace("@styx-tools/", "").to_string();
                                        display_name = name.clone();
                                    }
                                    if let Some(d) = val.get("description").and_then(|v| v.as_str())
                                    {
                                        desc = d.to_string();
                                    }
                                    if let Some(v) = val.get("version").and_then(|v| v.as_str()) {
                                        version = v.to_string();
                                    }
                                }

                            let is_installed = installed_servers
                                .iter()
                                .any(|s| s.name == name || s.name.contains(&dir_name));

                            discovered.push(json!({
                                "id": dir_name,
                                "name": name,
                                "display_name": display_name,
                                "version": version,
                                "description": desc,
                                "path": path.to_string_lossy().to_string(),
                                "is_installed": is_installed,
                                "has_docker": has_docker,
                            }));
                        }
                    }
                }
            }
    }

    (
        StatusCode::OK,
        Json(json!({ "success": true, "tools": discovered })),
    )
}

pub async fn install_tool(
    State(state): State<AppState>,
    Json(payload): Json<InstallToolRequest>,
) -> impl IntoResponse {
    let tool_name = payload.name.clone();
    let transport_type = payload.transport_type.unwrap_or_else(|| "http".to_string());
    let auto_start = payload.auto_start_container.unwrap_or(true);

    let container_name = format!(
        "styx-{}-connector",
        tool_name.replace('@', "").replace('/', "-")
    );

    // If auto_start is true and container is stopped, attempt to start it
    if auto_start
        && let Ok(containers) = state.docker.list_containers(true).await
            && let Some(c) = containers
                .iter()
                .find(|c| c.names.iter().any(|n| n.contains(&container_name)))
                && c.state != "running" && !c.status.to_lowercase().contains("up") {
                    let _ = state.docker.start_container(&c.id).await;
                    tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                }

    // Resolve port if path is provided
    let mut detected_port: u16 = 8080;
    if let Some(ref p) = payload.path {
        let pbuf = std::path::Path::new(p);
        let manifest_path = pbuf.join("manifest.json");
        if let Ok(c) = std::fs::read_to_string(&manifest_path)
            && let Ok(v) = serde_json::from_str::<Value>(&c)
            && let Some(p) = v.get("port").and_then(|x| x.as_u64())
        {
            detected_port = p as u16;
        }
    }

    // Resolve URL
    let resolved_url = if let Some(u) = payload.url {
        u
    } else {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_millis(1000))
            .build()
            .unwrap_or_default();

        let host_docker_url = format!("http://host.docker.internal:{}/mcp", detected_port);
        let local_url = format!("http://127.0.0.1:{}/mcp", detected_port);

        if client
            .get(format!("http://host.docker.internal:{}/health", detected_port))
            .send()
            .await
            .is_ok()
        {
            host_docker_url
        } else if client
            .get(format!("http://127.0.0.1:{}/health", detected_port))
            .send()
            .await
            .is_ok()
        {
            local_url
        } else {
            host_docker_url
        }
    };

    // Remove any previous registration for this tool to prevent duplicates
    if let Ok(existing) = state.db.list_mcp_servers().await {
        for s in existing {
            if s.name == tool_name {
                state.mcp.unregister_server(&s.id).await;
                let _ = state.db.delete_mcp_server(&s.id).await;
            }
        }
    }

    // Connect transport
    let transport = McpTransport::connect_http(&resolved_url);

    // Save server in DB
    let record = match state
        .db
        .add_mcp_server(
            &tool_name,
            &transport_type,
            None,
            None,
            None,
            None,
            Some(&resolved_url),
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

    // Register with MCP registry & perform handshake
    let empty_policies = HashMap::new();
    match state
        .mcp
        .register_server(&record.id, transport, &empty_policies)
        .await
    {
        Ok(tools) => {
            // Automatically ingest & install tool's shareable instructions into Styx memory
            let mut instructions_installed = false;
            let mut instructions_file = None;
            let mut instructions_content: Option<String> = None;

            if let Some(ref p) = payload.path {
                let pbuf = std::path::Path::new(p);
                for fname in &["instructions.md", "skill.md", "README.md"] {
                    let cand = pbuf.join(fname);
                    if cand.exists()
                        && let Ok(c) = std::fs::read_to_string(&cand)
                            && !c.trim().is_empty() {
                                instructions_content = Some(c);
                                break;
                            }
                }
            }

            if instructions_content.is_none() {
                let client = reqwest::Client::builder()
                    .timeout(std::time::Duration::from_millis(1500))
                    .build()
                    .unwrap_or_default();
                let inst_url = resolved_url.replace("/mcp", "/instructions");
                if let Ok(resp) = client.get(&inst_url).send().await
                    && let Ok(val) = resp.json::<Value>().await
                        && let Some(inst) = val.get("instructions").and_then(|i| i.as_str()) {
                            instructions_content = Some(inst.to_string());
                        }
            }

            if let Some(content) = instructions_content {
                let clean_name = tool_name.to_lowercase().replace('@', "").replace('/', "_");
                let rel_path = format!("skills/{}.md", clean_name);
                if state.memory.write_file(&rel_path, &content, None).await.is_ok() {
                    instructions_installed = true;
                    instructions_file = Some(rel_path);
                }
            }

            (
                StatusCode::OK,
                Json(json!({
                    "success": true,
                    "message": format!(
                        "Successfully registered {} with {} tools",
                        tool_name,
                        tools.len()
                    ),
                    "server": record,
                    "discovered_tools": tools,
                    "instructions_installed": instructions_installed,
                    "instructions_file": instructions_file
                })),
            )
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": format!(
                    "Handshake failed with MCP server at {}: {}",
                    resolved_url, e
                )
            })),
        ),
    }
}
