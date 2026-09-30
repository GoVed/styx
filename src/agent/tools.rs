use serde_json::Value;

use crate::db::Database;
use crate::mcp::{McpRegistry, ToolCallOutput};
use crate::memory::MemoryManager;
use crate::telemetry::TelemetryCollector;

pub struct AgentToolExecutor<'a> {
    pub db: &'a Database,
    pub memory: &'a MemoryManager,
    pub mcp: &'a McpRegistry,
    pub telemetry: &'a TelemetryCollector,
}

impl<'a> AgentToolExecutor<'a> {
    pub async fn execute_tool(&self, name: &str, args: Value) -> ToolCallOutput {
        match name {
            "read_memory" => {
                let path = args.get("path").and_then(|p| p.as_str()).unwrap_or("");
                match self.memory.read_file(path) {
                    Ok(content) => ToolCallOutput {
                        tool_name: name.to_string(),
                        success: true,
                        content,
                        is_error: false,
                    },
                    Err(e) => ToolCallOutput {
                        tool_name: name.to_string(),
                        success: false,
                        content: format!("Failed to read memory file: {}", e),
                        is_error: true,
                    },
                }
            }
            "search_memory" => {
                let query = args.get("query").and_then(|q| q.as_str()).unwrap_or("");
                let limit = args
                    .get("limit")
                    .and_then(|l| l.as_u64())
                    .unwrap_or(5) as usize;

                match self.memory.search(query, limit) {
                    Ok(results) => {
                        let json_str = serde_json::to_string_pretty(&results).unwrap_or_default();
                        ToolCallOutput {
                            tool_name: name.to_string(),
                            success: true,
                            content: json_str,
                            is_error: false,
                        }
                    }
                    Err(e) => ToolCallOutput {
                        tool_name: name.to_string(),
                        success: false,
                        content: format!("Memory search error: {}", e),
                        is_error: true,
                    },
                }
            }
            "write_memory" => {
                let path = args.get("path").and_then(|p| p.as_str()).unwrap_or("");
                let content = args.get("content").and_then(|c| c.as_str()).unwrap_or("");
                let section = args.get("section").and_then(|s| s.as_str());

                match self.memory.write_file(path, content, section).await {
                    Ok(()) => {
                        if path.contains("user_profile") {
                            let _ = self.db.set_setting("onboarding_completed", "true").await;
                        }
                        ToolCallOutput {
                            tool_name: name.to_string(),
                            success: true,
                            content: format!("Successfully updated memory file: {}", path),
                            is_error: false,
                        }
                    }
                    Err(e) => ToolCallOutput {
                        tool_name: name.to_string(),
                        success: false,
                        content: format!("Failed to write memory file: {}", e),
                        is_error: true,
                    },
                }
            }
            "complete_onboarding" => {
                let summary = args
                    .get("summary")
                    .and_then(|s| s.as_str())
                    .unwrap_or("Onboarding setup finished");
                let _ = self.db.set_setting("onboarding_completed", "true").await;
                ToolCallOutput {
                    tool_name: name.to_string(),
                    success: true,
                    content: format!(
                        "User enrollment and onboarding marked as completed in Styx. Summary: {}",
                        summary
                    ),
                    is_error: false,
                }
            }
            "exec_container_command" => {
                let command = args.get("command").and_then(|c| c.as_str()).unwrap_or("");
                let workdir = args.get("workdir").and_then(|w| w.as_str()).unwrap_or("/app");

                if command.trim().is_empty() {
                    return ToolCallOutput {
                        tool_name: name.to_string(),
                        success: false,
                        content: "Empty command provided".to_string(),
                        is_error: true,
                    };
                }

                let mut cmd = tokio::process::Command::new("bash");
                cmd.arg("-c").arg(command);
                if std::path::Path::new(workdir).exists() {
                    cmd.current_dir(workdir);
                }

                match tokio::time::timeout(tokio::time::Duration::from_secs(60), cmd.output()).await
                {
                    Ok(Ok(output)) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        let exit_code = output.status.code().unwrap_or(-1);

                        let mut result = format!("Exit Code: {}\n", exit_code);
                        if !stdout.is_empty() {
                            result.push_str(&format!("STDOUT:\n{}\n", stdout));
                        }
                        if !stderr.is_empty() {
                            result.push_str(&format!("STDERR:\n{}\n", stderr));
                        }
                        if stdout.is_empty() && stderr.is_empty() {
                            result.push_str("(Command finished with no output)");
                        }

                        ToolCallOutput {
                            tool_name: name.to_string(),
                            success: output.status.success(),
                            content: result,
                            is_error: !output.status.success(),
                        }
                    }
                    Ok(Err(e)) => ToolCallOutput {
                        tool_name: name.to_string(),
                        success: false,
                        content: format!("Failed to execute command inside container: {}", e),
                        is_error: true,
                    },
                    Err(_) => ToolCallOutput {
                        tool_name: name.to_string(),
                        success: false,
                        content: "Command execution timed out after 60 seconds".to_string(),
                        is_error: true,
                    },
                }
            }
            "get_system_telemetry" => {
                let snap = self
                    .telemetry
                    .collect_snapshot(0, 0, 0, "System".to_string(), 0, 0, 1)
                    .await;
                ToolCallOutput {
                    tool_name: name.to_string(),
                    success: true,
                    content: serde_json::to_string_pretty(&snap).unwrap_or_default(),
                    is_error: false,
                }
            }
            "inspect_image" => {
                let url = args
                    .get("url")
                    .or_else(|| args.get("image_url"))
                    .and_then(|u| u.as_str())
                    .unwrap_or("");
                let question = args
                    .get("question")
                    .or_else(|| args.get("prompt"))
                    .and_then(|q| q.as_str())
                    .unwrap_or("Describe what is shown in this image in detail, including key subjects, expressions, colors, and any text.");

                if url.trim().is_empty() {
                    return ToolCallOutput {
                        tool_name: name.to_string(),
                        success: false,
                        content: "Missing required 'url' parameter for inspect_image".to_string(),
                        is_error: true,
                    };
                }

                let model_cfg = match self.db.get_active_model_config().await {
                    Ok(Some(cfg)) => cfg,
                    _ => crate::db::ModelConfigRecord {
                        id: "default".to_string(),
                        name: "Local llama.cpp (128k Local GGUF)".to_string(),
                        provider: "docker_llamacpp".to_string(),
                        base_url: Some("http://localhost:8080/v1".to_string()),
                        api_key: None,
                        model_id: "/models/Qwen3.5-9B-UD-Q4_K_XL.gguf".to_string(),
                        context_length: 131072,
                        is_active: true,
                        extra_flags_json: None,
                        created_at: chrono::Utc::now().to_rfc3339(),
                    },
                };

                let base_url = model_cfg.base_url.as_deref().unwrap_or("http://localhost:8080/v1");
                let api_key = model_cfg.api_key.as_deref().unwrap_or("");
                let model_id = &model_cfg.model_id;

                match crate::router::vision::inspect_image_with_model(
                    base_url,
                    api_key,
                    model_id,
                    url,
                    question,
                )
                .await
                {
                    Ok(analysis) => ToolCallOutput {
                        tool_name: name.to_string(),
                        success: true,
                        content: analysis,
                        is_error: false,
                    },
                    Err(e) => ToolCallOutput {
                        tool_name: name.to_string(),
                        success: false,
                        content: format!("Image inspection error: {}", e),
                        is_error: true,
                    },
                }
            }
            _ => {
                // External MCP tool call
                match self.mcp.call_external_tool(name, args).await {
                    Ok(res) => res,
                    Err(e) => ToolCallOutput {
                        tool_name: name.to_string(),
                        success: false,
                        content: format!("External MCP tool execution error: {}", e),
                        is_error: true,
                    },
                }
            }
        }
    }
}
