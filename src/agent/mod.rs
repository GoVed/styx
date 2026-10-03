use anyhow::Result;
use serde_json::Value;
use tokio::sync::mpsc;

use crate::db::Database;
use crate::docker::DockerOrchestrator;
use crate::hitl::HitlGate;
use crate::mcp::policy::PolicyTier;
use crate::mcp::{McpRegistry, ToolCallOutput};
use crate::memory::MemoryManager;
use crate::router::openai::{ChatMessageParam, ToolParam};
use crate::router::MultiModelRouter;
use crate::telemetry::TelemetryCollector;

pub mod compression;
pub mod diagnosis;
pub mod execution;
pub mod prompt;
pub mod queue;
pub mod reasoning;
pub mod recovery;
pub mod tools;
pub mod types;

#[cfg(test)]
mod tests;

pub use reasoning::extract_thought_and_response;
pub use tools::AgentToolExecutor;
pub use types::AgentEvent;

pub struct AgentRunner {
    pub db: Database,
    pub memory: MemoryManager,
    pub mcp: McpRegistry,
    pub hitl: HitlGate,
    pub router: MultiModelRouter,
    pub telemetry: TelemetryCollector,
    pub docker: DockerOrchestrator,
}

impl AgentRunner {
    pub fn new(
        db: Database,
        memory: MemoryManager,
        mcp: McpRegistry,
        hitl: HitlGate,
        router: MultiModelRouter,
        telemetry: TelemetryCollector,
        docker: DockerOrchestrator,
    ) -> Self {
        Self {
            db,
            memory,
            mcp,
            hitl,
            router,
            telemetry,
            docker,
        }
    }

    #[allow(dead_code)]
    pub async fn probe_engine_ready(&self, base_url: Option<&str>, timeout_secs: u64) -> bool {
        diagnosis::probe_engine_ready(base_url, timeout_secs).await
    }

    pub async fn diagnose_inference_failure(
        &self,
        model_cfg: &crate::db::ModelConfigRecord,
        err_msg: &str,
    ) -> String {
        diagnosis::diagnose_inference_failure(&self.docker, model_cfg, err_msg).await
    }

    pub async fn execute_tool(&self, name: &str, args: Value) -> ToolCallOutput {
        let executor = AgentToolExecutor {
            db: &self.db,
            memory: &self.memory,
            mcp: &self.mcp,
            telemetry: &self.telemetry,
        };
        executor.execute_tool(name, args).await
    }

    #[allow(dead_code)]
    pub async fn run_turn(
        &self,
        session_id: &str,
        user_prompt: &str,
        mode: &str, // "chat" or "mission"
        event_tx: mpsc::Sender<AgentEvent>,
    ) -> Result<()> {
        self.run_turn_with_images(session_id, user_prompt, mode, None, event_tx).await
    }

    pub async fn run_turn_with_images(
        &self,
        session_id: &str,
        user_prompt: &str,
        mode: &str, // "chat" or "mission"
        images: Option<Vec<String>>,
        event_tx: mpsc::Sender<AgentEvent>,
    ) -> Result<()> {
        let is_mission_mode = mode == "mission";

        // Save user message to database with optional images
        let images_json = images.as_ref().and_then(|imgs| serde_json::to_string(imgs).ok());
        self.db
            .add_message_full(session_id, "user", user_prompt, None, None, images_json.as_deref())
            .await?;

        // Retrieve active model config
        let model_cfg = self
            .db
            .get_active_model_config()
            .await?
            .unwrap_or_else(|| crate::db::ModelConfigRecord {
                id: "default".to_string(),
                name: "Local vLLM (Qwen 3.5 9B AWQ)".to_string(),
                provider: "docker_vllm".to_string(),
                base_url: Some("http://localhost:8000/v1".to_string()),
                api_key: None,
                model_id: "QuantTrio/Qwen3.5-9B-AWQ".to_string(),
                context_length: 16384,
                is_active: true,
                extra_flags_json: None,
                created_at: chrono::Utc::now().to_rfc3339(),
            });

        // Check interactive recovery commands first
        if self
            .handle_interactive_recovery(session_id, user_prompt, &model_cfg, &event_tx)
            .await?
            .is_some()
        {
            return Ok(());
        }

        // Build system context
        let mut base_context = self.memory.build_system_context();

        // Actively retrieve relevant memory entities and semantic matches for the user prompt
        let active_mem = self.memory.fetch_active_context(user_prompt, images.as_deref());
        if let Some(ref block) = active_mem.formatted_prompt {
            base_context.push_str(block);
        }

        let reasoning_effort = self
            .db
            .get_setting("reasoning_effort")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "high".to_string());

        let is_vision_capable = tools::is_main_model_vision_capable(&model_cfg);
        let system_context = prompt::build_agent_system_prompt(
            &base_context,
            is_mission_mode,
            &reasoning_effort,
            is_vision_capable,
        );

        // Fetch past conversation messages
        let history = self.db.list_messages(session_id).await?;
        let mut chat_messages: Vec<ChatMessageParam> = Vec::new();

        chat_messages.push(ChatMessageParam {
            role: "system".to_string(),
            content: system_context,
            name: None,
            tool_call_id: None,
            tool_calls: None,
            images: None,
        });

        let hist_len = history.len();
        for (idx, msg) in history.iter().enumerate() {
            if msg.content.trim().is_empty()
                && msg.tool_calls.as_deref().unwrap_or("").trim().is_empty()
            {
                continue;
            }
            let llm_role = match msg.role.as_str() {
                "assistant" => "assistant",
                "system" => "system",
                "tool" => "tool",
                _ => "user",
            };
            let tool_calls_param = msg.tool_calls.as_deref().and_then(|tc| serde_json::from_str(tc).ok());
            let images_param = msg.images.as_deref().and_then(|imgs| serde_json::from_str(imgs).ok());

            let is_last_turn = idx + 1 == hist_len && msg.role == "user";
            let content = prompt::enrich_user_turn_content(user_prompt, &msg.content, is_last_turn);

            chat_messages.push(ChatMessageParam {
                role: llm_role.to_string(),
                content,
                name: None,
                tool_call_id: None,
                tool_calls: tool_calls_param,
                images: images_param,
            });
        }

        // Get tools that are NOT blocked
        let all_tools = self.mcp.list_tools().await;
        let mut available_tools = Vec::new();
        for t in all_tools {
            if t.policy != PolicyTier::Blocked {
                if is_vision_capable && (t.name == "inspect_image" || t.name == "describe_image" || t.name == "ocr_image") {
                    continue;
                }
                available_tools.push(ToolParam {
                    name: t.name,
                    description: t.description,
                    parameters: t.input_schema,
                });
            }
        }

        // Run multi-iteration turn execution loop
        let (raw_response, raw_thought) = self
            .execute_turn_loop(
                session_id,
                &model_cfg,
                chat_messages,
                available_tools,
                is_mission_mode,
                &event_tx,
            )
            .await?;

        // Separate reasoning and clean final response
        let (extracted_response, thought_opt) =
            extract_thought_and_response(&raw_response, &raw_thought);
        let final_response = prompt::sanitize_option_tags(&extracted_response);

        let saved_msg = self
            .db
            .add_message(
                session_id,
                "assistant",
                &final_response,
                thought_opt.as_deref(),
                None,
            )
            .await?;

        let _ = event_tx
            .send(AgentEvent::Done {
                message_id: saved_msg.id,
            })
            .await;

        if self.db.is_onboarded().await.unwrap_or(false) {
            let _ = event_tx.send(AgentEvent::OnboardingCompleted).await;
        }

        Ok(())
    }
}
