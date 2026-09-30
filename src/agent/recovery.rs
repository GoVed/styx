use anyhow::Result;
use tokio::sync::mpsc;

use crate::agent::diagnosis::{format_diagnosis_from_logs, probe_engine_ready};
use crate::agent::types::AgentEvent;
use crate::agent::AgentRunner;
use crate::db::ModelConfigRecord;
use crate::docker::engines::{DeployModelRequest, EngineKind, GpuVendor};

impl AgentRunner {
    pub async fn handle_interactive_recovery(
        &self,
        session_id: &str,
        user_prompt: &str,
        model_cfg: &ModelConfigRecord,
        event_tx: &mpsc::Sender<AgentEvent>,
    ) -> Result<Option<()>> {
        let prompt_lower = user_prompt.trim().to_lowercase();

        // 1. Re-deploy with safe 16k context window
        if prompt_lower.contains("re-deploy with 16k")
            || prompt_lower.contains("deploy with 16k")
            || prompt_lower.contains("switch to 16k")
            || prompt_lower.contains("16k context window")
            || prompt_lower.contains("safe 16k")
        {
            let start_txt = "🚀 **Re-deploying Qwen 3.5 9B with safe 16k context window...**\n\nStopping existing containers and applying optimal settings (16,384 tokens, FP8 KV cache, eager initialization)...\n\n";
            let _ = event_tx
                .send(AgentEvent::Token {
                    content: start_txt.to_string(),
                })
                .await;

            let deploy_req = DeployModelRequest {
                name: "qwen-3-5-9b".to_string(), engine: EngineKind::Vllm,
                hf_repo: "QuantTrio/Qwen3.5-9B-AWQ".to_string(), hf_token: None,
                context_window: Some(16384), speculative_model: None, num_speculative_tokens: None,
                gpu_devices: Some("all".to_string()), gpu_vendor: Some(GpuVendor::Auto),
                custom_image: None, tensor_parallel_size: Some(1), gpu_memory_utilization: Some(0.90),
                quantization: Some("awq".to_string()), port: Some(8000), enable_mtp: Some(false),
                enable_vision: Some(true), kv_cache_dtype: Some("fp8".to_string()), max_num_seqs: Some(2),
                extra_args: Some(vec!["--enforce-eager".to_string()]),
            };

            let deploy_result = self.docker.deploy_model_container(&deploy_req).await;
            if let Err(e) = deploy_result {
                let err_msg = format!("❌ Deployment failed: {}", e);
                let _ = event_tx
                    .send(AgentEvent::Token {
                        content: err_msg.clone(),
                    })
                    .await;
                let msg = self
                    .db
                    .add_message(session_id, "assistant", &err_msg, None, None)
                    .await?;
                let _ = event_tx.send(AgentEvent::Done { message_id: msg.id }).await;
                return Ok(Some(()));
            }

            let configs = self.db.list_model_configs().await.unwrap_or_default();
            let mut target_id = String::new();
            for c in configs {
                if c.model_id == "QuantTrio/Qwen3.5-9B-AWQ"
                    || c.base_url.as_deref().unwrap_or("").contains("8000")
                {
                    let _ = self
                        .db
                        .update_model_config(
                            &c.id,
                            "Local vLLM (Qwen 3.5 9B AWQ)",
                            "QuantTrio/Qwen3.5-9B-AWQ",
                            16384,
                        )
                        .await;
                    target_id = c.id;
                    break;
                }
            }
            if target_id.is_empty()
                && let Ok(rec) = self
                    .db
                    .add_model_config(
                        "Local vLLM (Qwen 3.5 9B AWQ)",
                        "docker_vllm",
                        Some("http://localhost:8000/v1"),
                        None,
                        "QuantTrio/Qwen3.5-9B-AWQ",
                        16384,
                        None,
                    )
                    .await
                {
                    target_id = rec.id;
                }
            if !target_id.is_empty() {
                let _ = self.db.set_active_model(&target_id).await;
            }

            let wait_txt = "🔄 Waiting for neural network weights to load into GPU memory...\n";
            let _ = event_tx
                .send(AgentEvent::Token {
                    content: wait_txt.to_string(),
                })
                .await;

            let ready = probe_engine_ready(Some("http://localhost:8000/v1"), 45).await;

            let finish_msg = if ready {
                "\n🎉 **Qwen 3.5 9B is now online and ready!**\n\n\
                Your 16k context window (16,384 tokens) is active and running with high-speed GPU acceleration (~28 tok/s).\n\n\
                What would you like to explore or do today?\n\n\
                <options>\n\
                <option>Help me organize my day</option>\n\
                <option>What are your capabilities?</option>\n\
                <option>Check GPU & memory status</option>\n\
                <option other=\"true\">Other (ask anything)...</option>\n\
                </options>"
            } else {
                "\n⏳ **Model container is still initializing.**\n\n\
                Model weights are loading. You can wait a moment and retry, or check the container logs.\n\n\
                <options>\n\
                <option>Wait 20 seconds and retry automatically</option>\n\
                <option>View recent error logs</option>\n\
                <option>Switch to Cloud AI</option>\n\
                </options>"
            };

            let _ = event_tx
                .send(AgentEvent::Token {
                    content: finish_msg.to_string(),
                })
                .await;

            let final_resp = format!("{}{}{}", start_txt, wait_txt, finish_msg);

            let msg = self
                .db
                .add_message(session_id, "assistant", &final_resp, None, None)
                .await?;
            let _ = event_tx.send(AgentEvent::Done { message_id: msg.id }).await;
            return Ok(Some(()));
        }

        // 2. Wait and retry
        if prompt_lower.contains("wait 20 seconds and retry")
            || prompt_lower.contains("wait and retry")
            || prompt_lower.contains("retry my question")
        {
            let _ = event_tx
                .send(AgentEvent::Token {
                    content: "⏳ Waiting for inference engine to complete initialization...\n"
                        .to_string(),
                })
                .await;

            let ready = probe_engine_ready(model_cfg.base_url.as_deref(), 25).await;

            if ready {
                let history = self.db.list_messages(session_id).await.unwrap_or_default();
                let prev_q = history.iter().rev().find(|m| {
                    m.role == "user"
                        && !m.content.to_lowercase().contains("wait")
                        && !m.content.to_lowercase().contains("re-deploy")
                        && !m.content.to_lowercase().contains("deploy")
                        && !m.content.to_lowercase().contains("view")
                        && !m.content.to_lowercase().contains("logs")
                });

                if let Some(q) = prev_q {
                    let announce = format!(
                        "✅ **Inference engine is online!** Proceeding with your question: *\"{}\"*\n\n",
                        q.content
                    );
                    let _ = event_tx.send(AgentEvent::Token { content: announce }).await;
                } else {
                    let ready_msg = "✅ **Inference engine is online and ready!** How can I assist you today?\n\n<options>\n<option>Help me organize my day</option>\n<option>What are your capabilities?</option>\n<option other=\"true\">Other...</option>\n</options>";
                    let _ = event_tx
                        .send(AgentEvent::Token {
                            content: ready_msg.to_string(),
                        })
                        .await;
                    let msg = self
                        .db
                        .add_message(session_id, "assistant", ready_msg, None, None)
                        .await?;
                    let _ = event_tx.send(AgentEvent::Done { message_id: msg.id }).await;
                    return Ok(Some(()));
                }
            } else {
                let diagnostic = crate::agent::diagnosis::diagnose_inference_failure(
                    &self.docker,
                    model_cfg,
                    "Engine warmup still in progress",
                )
                .await;
                let _ = event_tx
                    .send(AgentEvent::Token {
                        content: format!("\n{}", diagnostic),
                    })
                    .await;
                let msg = self
                    .db
                    .add_message(session_id, "assistant", &diagnostic, None, None)
                    .await?;
                let _ = event_tx.send(AgentEvent::Done { message_id: msg.id }).await;
                return Ok(Some(()));
            }
        }

        // 3. View recent logs
        if prompt_lower.contains("view recent")
            || prompt_lower.contains("view live")
            || prompt_lower.contains("view model logs")
            || prompt_lower.contains("view error logs")
        {
            let mut log_text = "No container logs available.".to_string();
            let mut is_running = false;
            let mut c_status = "offline".to_string();
            let mut model_name = "QuantTrio/Qwen3.5-9B-AWQ".to_string();
            if let Ok(containers) = self.docker.list_containers(true).await
                && let Some(c) = containers.into_iter().find(|c| {
                    c.is_styx_managed
                        || c.names
                            .iter()
                            .any(|n| n.contains("styx") || n.contains("vllm") || n.contains("qwen"))
                }) {
                    is_running = c.state == "running";
                    c_status = c.status.clone();
                    if let Some(m) = c.model_id {
                        model_name = m;
                    }
                    if let Ok(lines) = self.docker.get_container_logs(&c.id, 40).await
                        && !lines.is_empty() {
                            log_text = lines.join("\n");
                        }
                }

            let diagnosis = format_diagnosis_from_logs(
                &model_name,
                16384,
                &log_text,
                is_running,
                &c_status,
                &log_text,
            );

            let resp = format!(
                "{}\n\n\
                <details>\n\
                <summary>🔍 <strong>View Technical Diagnostic Logs (for advanced users)</strong></summary>\n\n\
                ```text\n{}\n```\n\
                </details>",
                diagnosis, log_text
            );

            let _ = event_tx
                .send(AgentEvent::Token {
                    content: resp.clone(),
                })
                .await;
            let msg = self
                .db
                .add_message(session_id, "assistant", &resp, None, None)
                .await?;
            let _ = event_tx.send(AgentEvent::Done { message_id: msg.id }).await;
            return Ok(Some(()));
        }

        // 4. Restart container
        if prompt_lower.contains("restart model container") {
            let mut restarted = false;
            if let Ok(containers) = self.docker.list_containers(true).await
                && let Some(c) = containers.into_iter().find(|c| {
                    c.is_styx_managed
                        || c.names
                            .iter()
                            .any(|n| n.contains("styx") || n.contains("vllm") || n.contains("qwen"))
                }) {
                    let _ = event_tx
                        .send(AgentEvent::Token {
                            content: format!(
                                "🔄 Restarting container `{}`...\n",
                                c.names.first().unwrap_or(&c.id)
                            ),
                        })
                        .await;
                    let _ = self.docker.restart_container(&c.id).await;
                    restarted = true;
                }

            if restarted {
                let ready = probe_engine_ready(model_cfg.base_url.as_deref(), 35).await;
                let resp = if ready {
                    "✅ **Container restarted and ready!** What would you like to do next?\n\n<options>\n<option>Help me organize my day</option>\n<option>What are your capabilities?</option>\n<option other=\"true\">Other...</option>\n</options>"
                } else {
                    "⏳ **Container is starting up.** Warmup is in progress.\n\n<options>\n<option>Wait 20 seconds and retry automatically</option>\n<option>View recent error logs</option>\n<option>Switch to Cloud AI</option>\n</options>"
                };
                let _ = event_tx.send(AgentEvent::Token { content: resp.to_string() }).await;
                let msg = self.db.add_message(session_id, "assistant", resp, None, None).await?;
                let _ = event_tx.send(AgentEvent::Done { message_id: msg.id }).await;
                return Ok(Some(()));
            }
        }

        // 5. Switch to Cloud AI
        if prompt_lower.contains("switch to cloud ai") {
            let configs = self.db.list_model_configs().await.unwrap_or_default();
            let cloud_opt = configs.into_iter().find(|c| {
                (c.provider == "anthropic" || c.provider == "google" || c.provider == "openai")
                    && c.api_key.as_deref().map(|k| !k.is_empty()).unwrap_or(false)
            });

            let resp = if let Some(cloud_cfg) = cloud_opt {
                let _ = self.db.set_active_model(&cloud_cfg.id).await;
                format!(
                    "☁️ Switched active model to **{}** ({})!\n\nHow can I help you today?\n\n<options>\n<option>Help me organize my day</option>\n<option>What are your capabilities?</option>\n<option other=\"true\">Other...</option>\n</options>",
                    cloud_cfg.name, cloud_cfg.model_id
                )
            } else {
                "☁️ **No Cloud AI Provider Configured**\n\nTo use Claude, Google Gemini, or OpenAI, please add your API key in **Settings Hub** or **Model Orchestrator**.\n\n<options>\n<option>Re-deploy with 16k context window (Recommended)</option>\n<option>Open Model Manager</option>\n<option>Wait 20 seconds and retry automatically</option>\n</options>".to_string()
            };

            let _ = event_tx
                .send(AgentEvent::Token {
                    content: resp.clone(),
                })
                .await;
            let msg = self
                .db
                .add_message(session_id, "assistant", &resp, None, None)
                .await?;
            let _ = event_tx.send(AgentEvent::Done { message_id: msg.id }).await;
            return Ok(Some(()));
        }

        Ok(None)
    }
}
