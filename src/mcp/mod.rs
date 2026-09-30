pub mod builtin;
pub mod policy;
pub mod transport;
pub mod types;

use anyhow::{bail, Context, Result};
use policy::PolicyTier;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{error, info};
use transport::McpTransport;
pub use types::*;

#[derive(Clone)]
pub struct McpRegistry {
    // Map server_id -> McpTransport
    transports: Arc<RwLock<HashMap<String, Arc<McpTransport>>>>,
    // Map tool_name -> server_id (None for built-in tools)
    tool_routes: Arc<RwLock<HashMap<String, Option<String>>>>,
    // Cached tool definitions
    cached_tools: Arc<RwLock<HashMap<String, McpTool>>>,
}

impl McpRegistry {
    pub fn new() -> Self {
        let builtins = builtin::get_builtin_tools();

        let mut routes = HashMap::new();
        let mut tools = HashMap::new();

        for t in builtins {
            routes.insert(t.name.clone(), None);
            tools.insert(t.name.clone(), t);
        }

        Self {
            transports: Arc::new(RwLock::new(HashMap::new())),
            tool_routes: Arc::new(RwLock::new(routes)),
            cached_tools: Arc::new(RwLock::new(tools)),
        }
    }

    pub async fn register_server(
        &self,
        server_id: &str,
        transport: McpTransport,
        override_policies: &HashMap<String, PolicyTier>,
    ) -> Result<Vec<McpTool>> {
        info!("Registering MCP server: {}", server_id);

        // Perform MCP handshake: initialize
        let init_params = json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "roots": { "listChanged": true },
                "sampling": {}
            },
            "clientInfo": {
                "name": "styx-harness",
                "version": "0.1.0"
            }
        });

        let init_res = transport.call_method("initialize", init_params).await?;
        info!("MCP Server {} initialized: {:?}", server_id, init_res);

        // Query tools/list
        let tools_res = transport.call_method("tools/list", json!({})).await?;
        let tools_array = tools_res
            .get("tools")
            .and_then(|v| v.as_array())
            .context("tools/list did not return a 'tools' array")?;

        let mut discovered = Vec::new();
        let transport_arc = Arc::new(transport);

        {
            let mut transports = self.transports.write().await;
            transports.insert(server_id.to_string(), transport_arc);
        }

        let mut routes = self.tool_routes.write().await;
        let mut cached = self.cached_tools.write().await;

        for item in tools_array {
            let name = item
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            if name.is_empty() {
                continue;
            }

            let description = item
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let input_schema = item
                .get("inputSchema")
                .cloned()
                .unwrap_or_else(|| json!({"type": "object"}));

            // Determine policy
            let (policy, risk) = if let Some(p) = override_policies.get(&name) {
                let (_, def_risk) = PolicyTier::default_for_tool(&name);
                (*p, def_risk.to_string())
            } else {
                let (p, r) = PolicyTier::default_for_tool(&name);
                (p, r.to_string())
            };

            let tool = McpTool {
                name: name.clone(),
                description,
                input_schema,
                server_id: Some(server_id.to_string()),
                policy,
                risk_level: risk,
            };

            routes.insert(name.clone(), Some(server_id.to_string()));
            cached.insert(name.clone(), tool.clone());
            discovered.push(tool);
        }

        info!(
            "MCP Server {} registered {} tools",
            server_id,
            discovered.len()
        );
        Ok(discovered)
    }

    pub async fn unregister_server(&self, server_id: &str) {
        let mut transports = self.transports.write().await;
        transports.remove(server_id);

        let mut routes = self.tool_routes.write().await;
        let mut cached = self.cached_tools.write().await;

        routes.retain(|_, s_id| s_id.as_deref() != Some(server_id));
        cached.retain(|_, tool| tool.server_id.as_deref() != Some(server_id));
    }

    pub async fn list_tools(&self) -> Vec<McpTool> {
        let cached = self.cached_tools.read().await;
        cached.values().cloned().collect()
    }

    pub async fn get_tool(&self, name: &str) -> Option<McpTool> {
        let cached = self.cached_tools.read().await;
        cached.get(name).cloned()
    }

    pub async fn update_tool_policy(&self, name: &str, policy: PolicyTier) -> Result<()> {
        let mut cached = self.cached_tools.write().await;
        if let Some(tool) = cached.get_mut(name) {
            tool.policy = policy;
            Ok(())
        } else {
            bail!("Tool not found: {}", name);
        }
    }

    pub async fn call_external_tool(&self, tool_name: &str, args: Value) -> Result<ToolCallOutput> {
        let server_id = {
            let routes = self.tool_routes.read().await;
            routes
                .get(tool_name)
                .cloned()
                .flatten()
                .context(format!("Tool '{}' has no active MCP server route", tool_name))?
        };

        let transport = {
            let transports = self.transports.read().await;
            transports
                .get(&server_id)
                .cloned()
                .context(format!("MCP Server '{}' not found", server_id))?
        };

        let params = json!({
            "name": tool_name,
            "arguments": args
        });

        match transport.call_method("tools/call", params).await {
            Ok(res) => {
                let is_error = res
                    .get("isError")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let content_str = if let Some(content_arr) = res.get("content").and_then(|v| v.as_array()) {
                    let mut parts = Vec::new();
                    for item in content_arr {
                        if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                            parts.push(text.to_string());
                        } else {
                            parts.push(item.to_string());
                        }
                    }
                    parts.join("\n")
                } else {
                    res.to_string()
                };

                Ok(ToolCallOutput {
                    tool_name: tool_name.to_string(),
                    success: !is_error,
                    content: content_str,
                    is_error,
                })
            }
            Err(e) => {
                error!("Error executing MCP tool '{}': {:?}", tool_name, e);
                Ok(ToolCallOutput {
                    tool_name: tool_name.to_string(),
                    success: false,
                    content: format!("Tool call error: {}", e),
                    is_error: true,
                })
            }
        }
    }
}
