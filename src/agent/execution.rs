use anyhow::Result;
use serde_json::{json, Value};
use std::time::Instant;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::agent::compression::{estimate_messages_tokens, prepare_auto_compressed_messages};
use crate::agent::types::AgentEvent;
use crate::agent::AgentRunner;
use crate::db::ModelConfigRecord;
use crate::hitl::{ApprovalDecision, ApprovalTicket};
use crate::mcp::policy::PolicyTier;
use crate::router::openai::{ChatMessageParam, ToolCallFunctionParam, ToolCallParam, ToolParam};
use crate::router::StreamChunk;

impl AgentRunner {
    pub async fn execute_turn_loop(
        &self,
        session_id: &str,
        model_cfg: &ModelConfigRecord,
        mut chat_messages: Vec<ChatMessageParam>,
        available_tools: Vec<ToolParam>,
        is_mission_mode: bool,
        event_tx: &mpsc::Sender<AgentEvent>,
    ) -> Result<(String, String)> {
        let max_iterations = if is_mission_mode { 15 } else { 10 };
        let mut iteration = 0;
        let mut accumulated_full_response = String::new();
        let mut accumulated_full_thought = String::new();

        let context_limit = if model_cfg.context_length > 0 {
            model_cfg.context_length as usize
        } else {
            16384
        };
        let target_generation_headroom = (context_limit / 4).clamp(1024, 8192);
        let max_prompt_budget = context_limit.saturating_sub(target_generation_headroom);

        chat_messages = prepare_auto_compressed_messages(chat_messages, max_prompt_budget);

        while iteration < max_iterations {
            iteration += 1;

            if estimate_messages_tokens(&chat_messages) > max_prompt_budget {
                chat_messages = prepare_auto_compressed_messages(chat_messages, max_prompt_budget);
            }

            let prompt_tokens = estimate_messages_tokens(&chat_messages);
            let remaining_kv = context_limit.saturating_sub(prompt_tokens + 32).max(512);
            let max_gen_tokens = Some(remaining_kv as u32);

            let mut rx = self
                .router
                .dispatch_stream(
                    &model_cfg.provider,
                    model_cfg.base_url.as_deref(),
                    model_cfg.api_key.as_deref(),
                    &model_cfg.model_id,
                    chat_messages.clone(),
                    available_tools.clone(),
                    max_gen_tokens,
                )
                .await;

            let mut current_turn_text = String::new();
            let mut current_turn_thought = String::new();
            let mut tool_calls_map: std::collections::HashMap<usize, (String, String, String)> =
                std::collections::HashMap::new();

            let mut token_count_turn = 0u64;
            let start_time = Instant::now();
            let mut turn_error: Option<String> = None;

            while let Some(chunk) = rx.recv().await {
                match chunk {
                    StreamChunk::Thought(th) => {
                        current_turn_thought.push_str(&th);
                        accumulated_full_thought.push_str(&th);
                        let _ = event_tx.send(AgentEvent::Thought { content: th }).await;
                    }
                    StreamChunk::Token(tok) => {
                        current_turn_text.push_str(&tok);
                        accumulated_full_response.push_str(&tok);
                        token_count_turn += 1;
                        self.telemetry.record_tokens(1);
                        let _ = event_tx.send(AgentEvent::Token { content: tok }).await;
                    }
                    StreamChunk::ToolCallDelta {
                        index,
                        id,
                        name,
                        arguments_delta,
                    } => {
                        let entry = tool_calls_map
                            .entry(index)
                            .or_insert_with(|| (String::new(), String::new(), String::new()));
                        if let Some(i) = id {
                            entry.0 = i;
                        }
                        if let Some(n) = name {
                            entry.1 = n;
                        }
                        entry.2.push_str(&arguments_delta);
                    }
                    StreamChunk::Done => {
                        break;
                    }
                    StreamChunk::Error(err) => {
                        let _ = event_tx.send(AgentEvent::Error { message: err.clone() }).await;
                        turn_error = Some(err);
                        break;
                    }
                }
            }

            if let Some(err) = turn_error {
                let diagnostic = self.diagnose_inference_failure(model_cfg, &err).await;
                if accumulated_full_response.trim().is_empty() {
                    let _ = event_tx
                        .send(AgentEvent::Token {
                            content: diagnostic.clone(),
                        })
                        .await;
                    accumulated_full_response = diagnostic;
                }
                break;
            }

            let elapsed_sec = start_time.elapsed().as_secs_f32();
            if elapsed_sec > 0.1 && token_count_turn > 0 {
                let rate = token_count_turn as f32 / elapsed_sec;
                self.telemetry.update_tok_rate(rate).await;
            }

            // If no tool calls were made, turn is complete
            if tool_calls_map.is_empty() {
                break;
            }

            // Process tool calls
            let mut sorted_calls: Vec<(usize, (String, String, String))> =
                tool_calls_map.into_iter().collect();
            sorted_calls.sort_by_key(|(idx, _)| *idx);

            for (_idx, (call_id, _name, _args)) in &mut sorted_calls {
                if call_id.is_empty() {
                    *call_id = format!("call_{}", Uuid::new_v4().simple());
                }
            }

            let tool_params: Vec<ToolCallParam> = sorted_calls
                .iter()
                .map(|(_idx, (call_id, name, args))| ToolCallParam {
                    id: call_id.clone(),
                    kind: "function".to_string(),
                    function: ToolCallFunctionParam {
                        name: name.clone(),
                        arguments: args.clone(),
                    },
                })
                .collect();

            chat_messages.push(ChatMessageParam {
                role: "assistant".to_string(),
                content: current_turn_text.clone(),
                name: None,
                tool_call_id: None,
                tool_calls: Some(tool_params),
                images: None,
            });

            for (_idx, (call_id, tool_name, args_str)) in sorted_calls {
                let parsed_args: Value =
                    serde_json::from_str(&args_str).unwrap_or_else(|_| json!({}));

                let _ = event_tx
                    .send(AgentEvent::ToolCallStarted {
                        id: call_id.clone(),
                        tool_name: tool_name.clone(),
                        arguments: parsed_args.clone(),
                    })
                    .await;

                // Check policy
                let tool_def = self.mcp.get_tool(&tool_name).await;
                let policy = tool_def
                    .as_ref()
                    .map(|t| t.policy)
                    .unwrap_or(PolicyTier::RequireApproval);
                let risk_level = tool_def
                    .as_ref()
                    .map(|t| t.risk_level.clone())
                    .unwrap_or_else(|| "HIGH".to_string());

                let (final_args, tool_permitted, rejection_reason) =
                    if policy == PolicyTier::RequireApproval {
                        self.handle_hitl_approval(
                            session_id,
                            &tool_name,
                            &args_str,
                            &parsed_args,
                            &risk_level,
                            event_tx,
                        )
                        .await
                    } else {
                        (parsed_args.clone(), true, None)
                    };

                let tool_start = Instant::now();
                let output = if tool_permitted {
                    let res = self.execute_tool(&tool_name, final_args.clone()).await;
                    let duration = tool_start.elapsed().as_millis() as u64;
                    let payload_str = serde_json::to_string(&final_args).unwrap_or_default();
                    let _ = self.db.record_audit_event(
                        Some(session_id), "tool_executed", Some(&tool_name), Some(&payload_str),
                        Some(if res.success { "success" } else { "error" }), Some(duration as i64),
                    ).await;
                    let _ = event_tx.send(AgentEvent::ToolCompleted {
                        id: call_id.clone(), tool_name: tool_name.clone(), success: res.success, output: res.content.clone(), duration_ms: duration,
                    }).await;

                    if tool_name == "complete_onboarding"
                        || (tool_name == "write_memory"
                            && final_args.get("path").and_then(|p| p.as_str()).map(|p| p.contains("user_profile")).unwrap_or(false))
                    {
                        let _ = self.db.set_setting("onboarding_completed", "true").await;
                        let _ = event_tx.send(AgentEvent::OnboardingCompleted).await;
                    }
                    res.content
                } else {
                    let duration = tool_start.elapsed().as_millis() as u64;
                    let reject_msg = format!(
                        "Tool execution rejected by human operator. Reason: {}",
                        rejection_reason.as_deref().unwrap_or("Action not permitted by user.")
                    );
                    let _ = self.db.record_audit_event(
                        Some(session_id), "tool_rejected", Some(&tool_name), Some(&args_str), Some("rejected"), Some(duration as i64),
                    ).await;
                    let _ = event_tx.send(AgentEvent::ToolCompleted {
                        id: call_id.clone(), tool_name: tool_name.clone(), success: false, output: reject_msg.clone(), duration_ms: duration,
                    }).await;
                    reject_msg
                };

                chat_messages.push(ChatMessageParam {
                    role: "tool".to_string(),
                    content: output,
                    name: Some(tool_name),
                    tool_call_id: Some(call_id),
                    tool_calls: None,
                    images: None,
                });
            }
        }

        // If tools executed but loop completed without final text response, force synthesis pass.
        if accumulated_full_response.trim().is_empty() {
            let mut final_messages = chat_messages.clone();
            final_messages.push(ChatMessageParam {
                role: "system".to_string(),
                content: "All tool actions are finished. Provide your direct response to the operator summarizing what was done and conclude with <options>.".to_string(),
                name: None, tool_call_id: None, tool_calls: None, images: None,
            });
            let mut rx = self.router.dispatch_stream(
                &model_cfg.provider, model_cfg.base_url.as_deref(), model_cfg.api_key.as_deref(),
                &model_cfg.model_id, final_messages, vec![], Some(1024),
            ).await;
            while let Some(chunk) = rx.recv().await {
                match chunk {
                    StreamChunk::Thought(th) => {
                        accumulated_full_thought.push_str(&th);
                        let _ = event_tx.send(AgentEvent::Thought { content: th }).await;
                    }
                    StreamChunk::Token(tok) => {
                        accumulated_full_response.push_str(&tok);
                        let _ = event_tx.send(AgentEvent::Token { content: tok }).await;
                    }
                    _ => {}
                }
            }
        }

        Ok((accumulated_full_response, accumulated_full_thought))
    }

    async fn handle_hitl_approval(
        &self,
        session_id: &str,
        tool_name: &str,
        args_str: &str,
        parsed_args: &Value,
        risk_level: &str,
        event_tx: &mpsc::Sender<AgentEvent>,
    ) -> (Value, bool, Option<String>) {
        let ticket_id = Uuid::new_v4().to_string();
        let ticket = ApprovalTicket {
            ticket_id: ticket_id.clone(),
            session_id: session_id.to_string(),
            tool_name: tool_name.to_string(),
            arguments: parsed_args.clone(),
            risk_level: risk_level.to_string(),
            explanation: Some(format!("Agent requested state-mutating tool '{}'", tool_name)),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let _ = self.db.create_approval_ticket(&ticket_id, session_id, tool_name, args_str, risk_level, ticket.explanation.as_deref()).await;
        let _ = event_tx.send(AgentEvent::ToolPendingApproval { ticket: ticket.clone() }).await;

        match self.hitl.submit_for_approval(ticket).await.await {
            Ok(ApprovalDecision::Approve) => {
                let _ = self.db.resolve_approval_ticket(&ticket_id, "approved", None).await;
                let _ = event_tx.send(AgentEvent::ToolApproved { ticket_id }).await;
                (parsed_args.clone(), true, None)
            }
            Ok(ApprovalDecision::ModifyPayload { new_arguments }) => {
                let mod_str = serde_json::to_string(&new_arguments).unwrap_or_default();
                let _ = self.db.resolve_approval_ticket(&ticket_id, "modified", Some(&mod_str)).await;
                let _ = event_tx.send(AgentEvent::ToolApproved { ticket_id }).await;
                (new_arguments, true, None)
            }
            Ok(ApprovalDecision::Reject { reason }) => {
                let _ = self.db.resolve_approval_ticket(&ticket_id, "rejected", None).await;
                let _ = event_tx.send(AgentEvent::ToolRejected { ticket_id, reason: reason.clone() }).await;
                (parsed_args.clone(), false, reason)
            }
            Err(_) => (parsed_args.clone(), false, Some("Approval channel canceled".to_string())),
        }
    }
}
