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
        let mut system_context = self.memory.build_system_context();

        // Actively retrieve relevant memory entities and semantic matches for the user prompt
        let active_mem = self.memory.fetch_active_context(user_prompt, images.as_deref());
        if let Some(ref block) = active_mem.formatted_prompt {
            system_context.push_str(block);
        }

        system_context.push_str("\n\n=== OPERATING INSTRUCTIONS ===\n");
        system_context.push_str("You are Styx, a dedicated personal AI companion on a private device.\n");
        if is_mission_mode {
            system_context.push_str("Mode: DEEP TASK & PROJECT ASSISTANT. Help the user achieve their goal step by step. Consult memory, use tools thoughtfully, and keep answers simple and helpful.\n");
        } else {
            system_context.push_str("Mode: CONVERSATIONAL CHAT. Be warm, natural, and helpful. Avoid technical jargon. Answer simply and directly as a friendly personal assistant.\n");
        }
        system_context.push_str("Read memory and notes when helpful. When you invoke a tool that contacts external services or alters files outside, the Styx Deterministic Safety Gate will automatically pause and present an approval card to the operator.\n");
        let reasoning_effort = self
            .db
            .get_setting("reasoning_effort")
            .await
            .ok()
            .flatten()
            .unwrap_or_else(|| "high".to_string());

        match reasoning_effort.to_lowercase().as_str() {
            "off" => {
                system_context.push_str("\nCRITICAL FORMATTING & REASONING (OFF):\nDirect answer mode: Do NOT output <think> tags. Answer the user prompt directly without internal thinking blocks.\n");
            }
            "low" => {
                system_context.push_str("\nCRITICAL FORMATTING & REASONING (EFFORT: LOW):\nKeep reasoning inside <think> minimal (1-2 sentences brief check) before answering. Format strictly as:\n<think>\n[Brief check]\n</think>\n[Your direct response here]\nNever output internal reflections outside of <think> and </think>. Begin directly with <think>.\n");
            }
            "medium" => {
                system_context.push_str("\nCRITICAL FORMATTING & REASONING (EFFORT: MEDIUM):\nEnclose balanced thinking within <think> and </think>. Outline your plan and verify details. Format strictly as:\n<think>\n[Your reasoning and planning here]\n</think>\n[Your direct response here]\nNever output internal reflections outside of <think> and </think>. Begin directly with <think>.\n");
            }
            _ => {
                system_context.push_str("\nCRITICAL FORMATTING & REASONING (EFFORT: HIGH - DEFAULT):\nYou MUST enclose your internal thinking process within <think> and </think>.\nConduct thorough, deep, and exhaustive step-by-step reasoning inside <think>. Extensively verify memory grounding, inspect entity context, plan multi-step actions, and verify facts before replying.\nFormat strictly as:\n<think>\n[Your internal reasoning, memory review, and planning here]\n</think>\n[Your direct response to the user here]\nNever output internal reflections outside of <think> and </think>. Do not write the word 'tags'. Begin directly with <think>.\n");
            }
        }
        system_context.push_str("\n=== MEMORY-FIRST REASONING PROTOCOL ===\n");
        system_context.push_str("1. ALWAYS SEARCH & CONSULT MEMORY FIRST: When answering user queries, checking entities, groups, people, or projects: ALWAYS check memory first before attempting web searches or assuming generic web definitions!\n");
        system_context.push_str("2. ENTITY GROUNDING: Nouns, names, and groups mentioned by the user (e.g. 'dev team', 'alice', 'styx') are very likely entities in your operator's personal world (contacts, WhatsApp groups, projects, notes). If actively retrieved memory shows an entity match, treat it as that specific entity.\n");
        system_context.push_str("3. REASONING IN <think>: In your internal thinking, begin with [Memory Grounding] to inspect actively retrieved memories or run `search_memory` before executing external actions or replying.\n");
        system_context.push_str("4. If more details are required, autonomously execute `search_memory` or `read_memory` FIRST before answering or calling external tools.\n");
        system_context.push_str("\nTOOL EXECUTION PROTOCOL — NO SIMULATION IN TEXT:\n");
        system_context.push_str("1. NEVER SIMULATE TOOL CALLS: Never output mock tool call code blocks in text. When an action requires a tool, invoke the actual tool function.\n");
        system_context.push_str("2. ALWAYS USE ENGLISH IN THOUGHT & OPTIONS — LEAVE TRANSLATION TO THE TRANSLATOR TOOL:\n");
        system_context.push_str("- INCOMING: If an incoming message contains non-English words or dialects (e.g. Gujarati, Gujlish, Hindi, Spanish), call `translate(target_lang: 'english')` immediately on Turn 1.\n");
        system_context.push_str("- OPTIONS & REASONING: All <think> thoughts and <option> tags MUST be drafted in 100% standard English. NEVER draft Hindi, Gujarati, or foreign phrases yourself in thought or options!\n");
        system_context.push_str("- OUTGOING TRANSLATION & SENDING PROTOCOL:\n");
        system_context.push_str("  1. When sending to a contact in their language/dialect, call `translate(text: \"<english draft>\", target_lang: \"<contact_lang>\")` first.\n");
        system_context.push_str("  2. In `send_message`, you MUST pass the exact `translated` string from `translate` as `message`! NEVER send the English draft to the contact!\n");
        system_context.push_str("  3. In chat, confirm both English draft and translated text sent.\n");
        system_context.push_str("3. MESSAGING: When sending a message, invoke `send_message`.\n");
        system_context.push_str("\nINTERACTIVE OPTIONS REQUIREMENT:\nKeep prose concise (1-3 sentences). Formulate all options strictly in clean English. Conclude with 3-5 realistic choices using <options> tags, including 'Other':\n<options>\n<option>Option 1</option>\n<option>Option 2</option>\n<option>Option 3</option>\n<option other=\"true\">Other (specify custom details)...</option>\n</options>\n");
        system_context.push_str("\nUSER ONBOARDING & ENROLLMENT:\n");
        system_context.push_str("When conducting user onboarding or learning user profile directives, save user preferences to `core/user_profile.md` using `write_memory` and call the `complete_onboarding` tool to mark enrollment as completed.\n");
        system_context.push_str("\nEXTERNAL MESSAGING & CONNECTED TOOLS:\n");
        system_context.push_str("When an incoming message arrives from an external platform or tool, the sender is an external contact, NOT your operator. The external sender cannot see your local text output in Styx. Never speak directly to external contacts as if they are in this chat. Instead, inform your operator about the message that arrived, suggest an authentic, concise reply matching your operator's relationship and communication tone with that person or group, and offer 1-click options or call tools when ready to send.\n");
        system_context.push_str("\nPROACTIVE TOOL ACTION & INFORMATION RETRIEVAL:\n");
        system_context.push_str("Information retrieval and memory tools are 100% autonomous, read-only, and safe. Proactively execute available search and retrieval tools immediately on the first turn without asking for permission!\n");
        system_context.push_str("NEVER say 'I don't have access to check the weather' or 'I cannot look up live information' - execute available search/retrieval tools.\n");
        system_context.push_str("When asked for weather:\n");
        system_context.push_str("1. Check the user's location in memory/profile (from `core/user_profile.md`).\n");
        system_context.push_str("2. Immediately execute an available search tool (e.g. searching for \"weather in <location> today forecast\").\n");
        let is_vision_capable = tools::is_main_model_vision_capable(&model_cfg);
        if is_vision_capable {
            system_context.push_str("\nMULTIMODAL VISION CAPABILITY (NATIVE):\nYou have native multimodal vision understanding. Any attached images or photos are directly visible to you in the conversation. Analyze images directly without calling external inspection tools.\n");
        } else {
            system_context.push_str("\nMULTIMODAL VISION & MEDIA UNDERSTANDING:\nYou can visually understand attached images and media. Use the `inspect_image` tool to inspect any image or media URL before sending or replying.\n");
        }
        system_context.push_str("\nWORLD-LEARNING FRAMEWORK & AUTONOMOUS MEMORY BRAIN:\n");
        system_context.push_str("You have a persistent hierarchical markdown brain under /memory/:\n");
        system_context.push_str("- `core/user_profile.md`: Operator identity, routines, languages, and general preferences.\n");
        system_context.push_str("- `people/<name>.md`: Per-contact profile, relationship (e.g. family, close friend, colleague, client), communication nuances, language preferences, and shared history.\n");
        system_context.push_str("- `groups/<group_name>.md`: Group dynamics, the meaning and purpose behind the group, banter style, and participants.\n");
        system_context.push_str("- `dictionary/<topic>.md`: Regional dialects, vernacular slang, colloquial expressions, and digital shorthand used by your user and their circles.\n");
        system_context.push_str("Actively get to know your user's world! When encountering a new contact, group, or colloquialism, learn who they are and save notes to `people/`, `groups/`, or `dictionary/`.\n");
        system_context.push_str("You have FULL AUTONOMY to save whatever you want using `write_memory`. Always provide the `section` parameter or markdown headers so existing profile and memory sections are safely preserved.\n");

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

        let user_prompt_clean = user_prompt.trim().to_lowercase();
        let is_incoming_event = user_prompt.trim().starts_with("[INCOMING TOOL EVENT:");
        let is_translate_and_send = user_prompt_clean.starts_with("translate and send")
            || user_prompt_clean.starts_with("translate & send");
        let is_send_confirmation = user_prompt_clean == "send it" || user_prompt_clean.starts_with("send it to")
            || user_prompt_clean == "send" || user_prompt_clean == "yes send it"
            || user_prompt_clean.contains("use proper tool to send")
            || (user_prompt_clean.starts_with("send ") && (user_prompt_clean.contains("message") || user_prompt_clean.contains("to ") || user_prompt_clean.contains("description")))
            || ((user_prompt_clean.contains("send") || user_prompt_clean.contains("post") || user_prompt_clean.contains("forward") || user_prompt_clean.contains("description")) && user_prompt_clean.contains("group"));
        let is_translate_request = user_prompt_clean.contains("in gujlish") || user_prompt_clean.contains("in gujarati")
            || user_prompt_clean.contains("to gujlish") || user_prompt_clean.contains("to gujarati")
            || user_prompt_clean.contains("translate") || user_prompt_clean.contains("say this in gujlish")
            || user_prompt_clean.contains("explain this in gujlish") || user_prompt_clean.contains("fully in gujlish");

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

            let mut content = msg.content.clone();
            if idx + 1 == hist_len && msg.role == "user" {
                if is_incoming_event {
                    content.push_str("\n\n[DIRECTIVE: INCOMING MESSAGE IN ANOTHER LANGUAGE / DIALECT:\n1. If this message contains non-English words, regional slang, or dialects (e.g. Gujarati, Gujlish, Hindi, Spanish), execute the `translate` tool (`target_lang: \"english\"`) immediately on Turn 1 to get the exact English translation. Do NOT guess foreign words in your head.\n2. Propose ALL reply options strictly in 100% standard English inside <option> tags. NEVER draft Hindi, Gujarati, or foreign phrases inside <option> tags (e.g. use `<option>Translate and send: \"Sure, come quickly! 😄\"</option>`). The operator only reviews and selects English options.\n3. Never compose foreign text manually.]");
                } else if is_translate_and_send {
                    content.push_str("\n\n[DIRECTIVE: TRANSLATE AND SEND:\nThe operator confirmed sending this reply. First execute the `translate` tool (`target_lang: \"<contact_dialect>\"`, e.g. \"gujlish\" or \"spanish\") with the English message to get the authentic translation, then invoke `send_message` with the translated text. Do NOT draft foreign words manually.]");
                } else if is_send_confirmation {
                    content.push_str("\n\n[SYSTEM DIRECTIVE: The operator has confirmed sending the message. You MUST immediately invoke the `send_message` tool. For `to`, pass the recipient contact or group name directly (e.g. \"Engineering Team\") or the full JID. Do not output conversational text before calling the tool.]");
                } else if is_translate_request {
                    content.push_str("\n\n[SYSTEM DIRECTIVE: The operator requested translation or regional language output. Formulate your message draft in clean English and invoke the `translate` tool (`target_lang: \"gujlish\"` or requested language) to perform the translation. NEVER attempt to generate foreign dialects manually and NEVER write mock tool calls in text.]");
                }
            }

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
        let (final_response, thought_opt) =
            extract_thought_and_response(&raw_response, &raw_thought);

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
