use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::state::AppState;

#[derive(Deserialize)]
pub struct InspectToolRequest {
    pub path: Option<String>,
    pub manifest: Option<Value>,
}

pub async fn inspect_tool(
    State(state): State<AppState>,
    Json(payload): Json<InspectToolRequest>,
) -> impl IntoResponse {
    let host_home = std::env::var("HOST_HOME").unwrap_or_else(|_| std::env::var("HOME").unwrap_or_else(|_| "/root".to_string()));
    let candidate_paths = if let Some(ref p) = payload.path {
        let trimmed = p.trim().trim_end_matches('/');
        let base_name = std::path::Path::new(trimmed)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        vec![
            trimmed.to_string(),
            format!("/tools/{}", base_name),
            format!("../syndae_tools/{}", base_name),
            format!("{}/Projects/syndae_tools/{}", host_home, base_name),
            format!("/app/syndae_tools/{}", base_name),
        ]
    } else {
        vec![
            "/tools".to_string(),
            format!("{}/Projects/syndae_tools", host_home),
        ]
    };

    let mut found_dir: Option<std::path::PathBuf> = None;
    for cand in &candidate_paths {
        let p = std::path::Path::new(cand);
        if p.is_dir() {
            found_dir = Some(p.to_path_buf());
            break;
        }
    }

    let mut tool_name = "tool".to_string();
    let mut display_name = "Local Tool".to_string();
    let mut description = "Local Micro-Daemon & MCP Tool for Syndae Agent OS".to_string();
    let mut version = "1.0.0".to_string();
    let mut has_docker = false;
    let mut has_compose = false;
    let mut container_name = String::new();
    let mut port: u16 = 8765;
    let mut known_tools: Vec<Value> = Vec::new();
    let mut skill_file: Option<String> = None;

    if let Some(ref dir) = found_dir {
        let dir_basename = dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        tool_name = dir_basename.clone();
        display_name = dir_basename.clone();
        container_name = format!("syndae-{}-connector", dir_basename);

        // 1. Check manifest.json
        let manifest_path = dir.join("manifest.json");
        if manifest_path.exists() {
            if let Ok(c) = std::fs::read_to_string(&manifest_path)
                && let Ok(v) = serde_json::from_str::<Value>(&c) {
                    if let Some(n) = v.get("name").and_then(|x| x.as_str()) {
                        tool_name = n.to_string();
                    }
                    if let Some(dn) = v.get("display_name").and_then(|x| x.as_str()) {
                        display_name = dn.to_string();
                    }
                    if let Some(d) = v.get("description").and_then(|x| x.as_str()) {
                        description = d.to_string();
                    }
                    if let Some(ver) = v.get("version").and_then(|x| x.as_str()) {
                        version = ver.to_string();
                    }
                    if let Some(cn) = v.get("container_name").and_then(|x| x.as_str()) {
                        container_name = cn.to_string();
                    }
                    if let Some(p) = v.get("port").and_then(|x| x.as_u64()) {
                        port = p as u16;
                    }
                    if let Some(tools_arr) = v.get("tools").and_then(|x| x.as_array()) {
                        known_tools = tools_arr.clone();
                    }
                    if let Some(sf) = v.get("skill_file").and_then(|x| x.as_str()) {
                        skill_file = Some(sf.to_string());
                    }
                }
        } else {
            // 2. Check package.json
            let pkg_path = dir.join("package.json");
            if pkg_path.exists()
                && let Ok(c) = std::fs::read_to_string(&pkg_path)
                    && let Ok(v) = serde_json::from_str::<Value>(&c) {
                        if let Some(n) = v.get("name").and_then(|x| x.as_str()) {
                            tool_name = n.replace("@syndae-tools/", "").to_string();
                            display_name = tool_name.clone();
                            container_name = format!("syndae-{}-connector", tool_name);
                        }
                        if let Some(d) = v.get("description").and_then(|x| x.as_str()) {
                            description = d.to_string();
                        }
                        if let Some(ver) = v.get("version").and_then(|x| x.as_str()) {
                            version = ver.to_string();
                        }
                    }
        }

        let compose_path = dir.join("docker-compose.yml");
        if compose_path.exists() {
            has_compose = true;
            has_docker = true;
        }

        let dockerfile_path = dir.join("Dockerfile");
        if dockerfile_path.exists() {
            has_docker = true;
        }
    } else if let Some(ref manifest) = payload.manifest {
        if let Some(n) = manifest.get("name").and_then(|x| x.as_str()) {
            tool_name = n.replace("@syndae-tools/", "").to_string();
            display_name = tool_name.clone();
            container_name = format!("syndae-{}-connector", tool_name);
        }
        if let Some(dn) = manifest.get("display_name").and_then(|x| x.as_str()) {
            display_name = dn.to_string();
        }
        if let Some(d) = manifest.get("description").and_then(|x| x.as_str()) {
            description = d.to_string();
        }
        if let Some(ver) = manifest.get("version").and_then(|x| x.as_str()) {
            version = ver.to_string();
        }
        if let Some(cn) = manifest.get("container_name").and_then(|x| x.as_str()) {
            container_name = cn.to_string();
        }
        if let Some(p) = manifest.get("port").and_then(|x| x.as_u64()) {
            port = p as u16;
        }
        if let Some(tools_arr) = manifest.get("tools").and_then(|x| x.as_array()) {
            known_tools = tools_arr.clone();
        }
        if let Some(sf) = manifest.get("skill_file").and_then(|x| x.as_str()) {
            skill_file = Some(sf.to_string());
        }
    }

    if container_name.is_empty() {
        container_name = format!("syndae-{}-connector", tool_name);
    }

    // Check Docker status
    let mut container_running = false;
    let mut container_exists = false;
    let mut container_status = "not_found".to_string();

    if let Ok(containers) = state.docker.list_containers(true).await {
        for c in containers {
            if c.names.iter().any(|n| n.contains(&container_name)) {
                container_exists = true;
                container_status = c.status.clone();
                if c.state == "running" || c.status.to_lowercase().contains("up") {
                    container_running = true;
                }
                break;
            }
        }
    }

    // Check HTTP health status
    let mut endpoint_healthy = false;
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(1500))
        .build()
        .unwrap_or_default();

    let health_urls = [
        format!("http://127.0.0.1:{}/health", port),
        format!("http://host.docker.internal:{}/health", port),
        format!("http://localhost:{}/health", port),
    ];

    let mut working_base_url = None;
    for u in &health_urls {
        if let Ok(resp) = http_client.get(u).send().await
            && resp.status().is_success() {
                endpoint_healthy = true;
                let parsed_url = reqwest::Url::parse(u).ok();
                if let Some(p) = parsed_url {
                    working_base_url = Some(format!(
                        "{}://{}:{}",
                        p.scheme(),
                        p.host_str().unwrap_or("127.0.0.1"),
                        port
                    ));
                }
                break;
            }
    }

    // Check if already registered in Syndae DB
    let is_registered = state
        .db
        .list_mcp_servers()
        .await
        .unwrap_or_default()
        .into_iter()
        .any(|s| s.name == tool_name || s.name.contains(&tool_name));

    // If known_tools is empty and endpoint is healthy, dynamically fetch tools from /mcp via JSON-RPC tools/list
    if known_tools.is_empty() && endpoint_healthy {
        let mcp_endpoint = if let Some(ref base) = working_base_url {
            format!("{}/mcp", base)
        } else {
            format!("http://127.0.0.1:{}/mcp", port)
        };

        let list_req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        });

        if let Ok(resp) = http_client.post(&mcp_endpoint).json(&list_req).send().await
            && let Ok(val) = resp.json::<Value>().await
                && let Some(tools) = val
                    .get("result")
                    .and_then(|r| r.get("tools"))
                    .and_then(|t| t.as_array())
                {
                    known_tools = tools.clone();
                }
    }

    let target_mcp_url = if let Some(base) = working_base_url.as_ref() {
        format!("{}/mcp", base)
    } else {
        format!("http://host.docker.internal:{}/mcp", port)
    };

    // Check for shareable instructions on disk or via tool daemon GET /instructions
    let mut instructions_content: Option<String> = None;
    if let Some(ref dir) = found_dir {
        for fname in &["instructions.md", "skill.md", "README.md"] {
            let cand = dir.join(fname);
            if cand.exists()
                && let Ok(c) = std::fs::read_to_string(&cand)
                    && !c.trim().is_empty() {
                        instructions_content = Some(c);
                        skill_file = Some(fname.to_string());
                        break;
                    }
        }
    }

    if instructions_content.is_none() && endpoint_healthy {
        let fallback_base = format!("http://127.0.0.1:{}", port);
        let base_inst = working_base_url.as_deref().unwrap_or(&fallback_base);
        let inst_url = format!("{}/instructions", base_inst);
        if let Ok(resp) = http_client.get(&inst_url).send().await
            && let Ok(val) = resp.json::<Value>().await
                && let Some(inst) = val.get("instructions").and_then(|i| i.as_str()) {
                    instructions_content = Some(inst.to_string());
                    skill_file = Some("instructions.md".to_string());
                }
    }

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "tool": {
                "name": tool_name,
                "display_name": display_name,
                "version": version,
                "description": description,
                "path": found_dir
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| payload.path.unwrap_or_default()),
                "has_docker": has_docker,
                "has_compose": has_compose,
                "container_name": container_name,
                "container_exists": container_exists,
                "container_running": container_running,
                "container_status": container_status,
                "container_healthy": endpoint_healthy,
                "port": port,
                "recommended_transport": "http",
                "recommended_url": target_mcp_url,
                "is_registered": is_registered,
                "tools": known_tools,
                "skill_file": skill_file.unwrap_or_else(|| format!("skills/{}.md", tool_name)),
                "has_instructions": instructions_content.is_some(),
                "instructions": instructions_content
            }
        })),
    )
}
