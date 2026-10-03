mod agent;
mod api;
mod config;
mod db;
mod docker;
mod hitl;
mod mcp;
mod memory;
mod router;
mod state;
mod telemetry;

use anyhow::Result;
use config::AppConfig;
use db::Database;
use docker::DockerOrchestrator;
use hitl::{HitlBroadcastEvent, HitlGate};
use mcp::transport::McpTransport;
use mcp::McpRegistry;
use memory::search::MemorySearchIndex;
use memory::MemoryManager;
use router::MultiModelRouter;
use serde_json::json;
use state::AppState;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use telemetry::TelemetryCollector;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "syndae=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env();
    info!("Starting Syndae Personal AI OS Runtime");
    info!("Host: {}:{}", config.host, config.port);
    info!("Memory directory: {:?}", config.memory_dir);
    info!("Database path: {:?}", config.db_path);

    // Initialize Database
    let db = Database::init(&config.db_path)?;
    info!("SQLite persistence initialized at {:?}", config.db_path);

    // Initialize Tantivy Search Index
    let search_index = Arc::new(MemorySearchIndex::new(&config.tantivy_dir)?);
    let _ = search_index.sync_all_files(&config.memory_dir).await;
    info!("Tantivy BM25 search engine initialized");

    // Initialize Memory Manager
    let memory = MemoryManager::new(config.memory_dir.clone(), search_index);
    let _ = memory.ensure_directories();

    // Initialize Docker Controller via Bollard
    let docker = DockerOrchestrator::new()?;
    info!("Bollard Docker Engine client connected via socket");

    // Initialize MCP Registry and Builtin Tools
    let mcp = McpRegistry::new();
    info!("Model Context Protocol (MCP) tool registry initialized with built-in tools");

    // Initialize Deterministic HITL Gate
    let hitl = HitlGate::new();

    // Initialize Multi-Model Router
    let router = MultiModelRouter::new();

    // Initialize System Telemetry
    let telemetry = TelemetryCollector::new();

    // Connect any registered MCP servers saved in database
    if let Ok(servers) = db.list_mcp_servers().await {
        for s in servers {
            if !s.enabled {
                continue;
            }
            let server_id = s.id.clone();
            let mcp_clone = mcp.clone();
            tokio::spawn(async move {
                let transport_res = match s.transport_type.as_str() {
                    "stdio" => {
                        let cmd = s.command.unwrap_or_default();
                        let args: Vec<String> = s
                            .args_json
                            .and_then(|a| serde_json::from_str(&a).ok())
                            .unwrap_or_default();
                        let env: Vec<(String, String)> = s
                            .env_json
                            .and_then(|e| serde_json::from_str::<HashMap<String, String>>(&e).ok())
                            .unwrap_or_default()
                            .into_iter()
                            .collect();
                        McpTransport::connect_stdio(&cmd, &args, &env).await
                    }
                    "unix_socket" => {
                        let path = s.socket_path.unwrap_or_default();
                        McpTransport::connect_unix(&path).await
                    }
                    "http_sse" | "http" => {
                        let url = s.url.unwrap_or_default();
                        Ok(McpTransport::connect_http(&url))
                    }
                    _ => Err(anyhow::anyhow!("Unknown transport")),
                };

                if let Ok(t) = transport_res {
                    let empty_map = HashMap::new();
                    let _ = mcp_clone.register_server(&server_id, t, &empty_map).await;
                }
            });
        }
    }

    // Build AppState
    let state = AppState::new(
        config.clone(),
        db.clone(),
        memory.clone(),
        docker.clone(),
        mcp.clone(),
        hitl.clone(),
        router.clone(),
        telemetry.clone(),
    );

    // Auto-sync active model from running local engine if available
    let db_for_sync = db.clone();
    tokio::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        crate::api::models::auto_sync_active_local_model(&db_for_sync).await;
    });

    // Spawn Background Telemetry Broadcast Loop
    let state_for_telemetry = state.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
        loop {
            interval.tick().await;

            let containers = state_for_telemetry
                .docker
                .list_containers(false)
                .await
                .unwrap_or_default();
            let mcp_tools = state_for_telemetry.mcp.list_tools().await;
            let pending = state_for_telemetry.hitl.list_pending_tickets().await.len();
            let active_model = state_for_telemetry
                .db
                .get_active_model_config()
                .await
                .ok()
                .flatten()
                .map(|m| format!("{}: {}", m.name, m.model_id))
                .unwrap_or_else(|| "Local vLLM (Docker)".to_string());
            let queue_status = state_for_telemetry.turn_queue.get_status().await;

            let snapshot = state_for_telemetry
                .telemetry
                .collect_snapshot(
                    containers.len(),
                    mcp_tools.len(),
                    pending,
                    active_model,
                    queue_status.queued_count,
                    queue_status.active_count,
                    queue_status.max_concurrent_turns,
                )
                .await;

            let msg = json!({
                "topic": "telemetry",
                "payload": snapshot
            });

            let _ = state_for_telemetry.ws_broadcast.send(msg.to_string());
        }
    });

    // Spawn Background HITL Event Forwarding Loop
    let state_for_hitl = state.clone();
    let mut hitl_rx = state.hitl.subscribe();
    tokio::spawn(async move {
        while let Ok(evt) = hitl_rx.recv().await {
            match evt {
                HitlBroadcastEvent::TicketCreated { ticket } => {
                    let msg = json!({
                        "topic": "approval_ticket",
                        "payload": ticket
                    });
                    let _ = state_for_hitl.ws_broadcast.send(msg.to_string());
                }
                HitlBroadcastEvent::TicketResolved {
                    ticket_id,
                    decision,
                    modified_arguments,
                } => {
                    let msg = json!({
                        "topic": "approval_resolved",
                        "payload": {
                            "ticket_id": ticket_id,
                            "decision": decision,
                            "modified_arguments": modified_arguments
                        }
                    });
                    let _ = state_for_hitl.ws_broadcast.send(msg.to_string());
                }
            }
        }
    });

    // Build Axum Router
    let app = api::build_router(state);

    let bind_addr: SocketAddr = format!("{}:{}", config.host, config.port)
        .parse()
        .expect("Invalid bind address");

    println!(
        r#"
========================================================================
   ███████╗██╗   ██╗███╗   ██╗██████╗   █████╗ ███████╗
   ██╔════╝╚██╗ ██╔╝████╗  ██║██╔══██╗ ██╔══██╗██╔════╝
   ███████╗ ╚████╔╝ ██╔██╗ ██║██║  ██║ ███████║█████╗  
   ╚════██║  ╚██╔╝  ██║╚██╗██║██║  ██║ ██╔══██║██╔══╝  
   ███████║   ██║   ██║ ╚████║██████╔╝ ██║  ██║███████╗
   ╚══════╝   ╚═╝   ╚═╝  ╚═══╝╚═════╝  ╚═╝  ╚═╝╚══════╝
   AGENT HARNESS, MODEL MANAGER & MISSION CONTROL DASHBOARD
========================================================================
   ▶ Mission Control UI:    http://{}:{}
   ▶ WebSocket Feed:        ws://{}:{}/ws
   ▶ REST API Endpoints:    http://{}:{}/api/*
   ▶ Memory Directory:      {}
   ▶ Docker Engine:         /var/run/docker.sock
   ▶ Tantivy Search:        Enabled (BM25)
   ▶ Deterministic HITL:    ARMED (Zero-Timeout)
========================================================================
"#,
        config.host,
        config.port,
        config.host,
        config.port,
        config.host,
        config.port,
        config.memory_dir.display(),
    );

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
