use anyhow::{bail, Context, Result};
use futures::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Instant;
use tokio::sync::mpsc;

use super::openai::{ChatMessageParam, HandshakeResult, ToolParam};
use super::StreamChunk;

#[derive(Clone)]
pub struct AnthropicClient {
    client: Client,
    api_key: String,
    model: String,
    reasoning_effort: Option<String>,
}

impl AnthropicClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self::new_with_reasoning(api_key, model, None)
    }

    pub fn new_with_reasoning(
        api_key: String,
        model: String,
        reasoning_effort: Option<String>,
    ) -> Self {
        Self {
            client: Client::builder().build().unwrap_or_default(),
            api_key,
            model,
            reasoning_effort,
        }
    }

    pub async fn test_connection(&self) -> HandshakeResult {
        let start = Instant::now();
        if self.api_key.is_empty() {
            return HandshakeResult {
                success: false,
                message: "Anthropic API key is empty".to_string(),
                latency_ms: 0,
                models: Vec::new(),
            };
        }

        let url = "https://api.anthropic.com/v1/messages";
        let body = json!({
            "model": self.model,
            "max_tokens": 1,
            "messages": [{"role": "user", "content": "ping"}]
        });

        match self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => {
                let latency_ms = start.elapsed().as_millis() as u64;
                HandshakeResult {
                    success: true,
                    message: format!("Anthropic connection verified with model {}", self.model),
                    latency_ms,
                    models: vec![self.model.clone()],
                }
            }
            Ok(resp) => {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                HandshakeResult {
                    success: false,
                    message: format!("Anthropic API error {}: {}", status, text),
                    latency_ms: start.elapsed().as_millis() as u64,
                    models: Vec::new(),
                }
            }
            Err(e) => HandshakeResult {
                success: false,
                message: format!("Anthropic request failed: {}", e),
                latency_ms: start.elapsed().as_millis() as u64,
                models: Vec::new(),
            },
        }
    }

    pub async fn stream_chat(
        &self,
        messages: Vec<ChatMessageParam>,
        tools: Vec<ToolParam>,
        tx: mpsc::Sender<StreamChunk>,
        max_tokens: Option<u32>,
    ) -> Result<()> {
        let url = "https://api.anthropic.com/v1/messages";

        // Convert messages to Anthropic format
        let mut ant_messages = Vec::new();
        let mut system_prompt = String::new();

        for m in messages {
            if m.role == "system" {
                if !system_prompt.is_empty() {
                    system_prompt.push_str("\n\n");
                }
                system_prompt.push_str(&m.content);
            } else if m.role == "tool" {
                // Tool result in Anthropic
                ant_messages.push(json!({
                    "role": "user",
                    "content": [{
                        "type": "tool_result",
                        "tool_use_id": m.tool_call_id.unwrap_or_default(),
                        "content": m.content
                    }]
                }));
            } else if m.role == "assistant" && m.tool_calls.is_some() {
                let mut content_blocks = Vec::new();
                if !m.content.trim().is_empty() {
                    content_blocks.push(json!({
                        "type": "text",
                        "text": m.content
                    }));
                }
                for tc in m.tool_calls.unwrap_or_default() {
                    let input_val: Value = serde_json::from_str(&tc.function.arguments).unwrap_or(json!({}));
                    content_blocks.push(json!({
                        "type": "tool_use",
                        "id": tc.id,
                        "name": tc.function.name,
                        "input": input_val
                    }));
                }
                ant_messages.push(json!({
                    "role": "assistant",
                    "content": content_blocks
                }));
            } else {
                ant_messages.push(json!({
                    "role": m.role,
                    "content": m.content
                }));
            }
        }

        let max_tok = max_tokens.unwrap_or(4096).min(8192);

        let mut body = json!({
            "model": self.model,
            "max_tokens": max_tok,
            "messages": ant_messages,
            "stream": true,
        });

        if !system_prompt.is_empty() {
            body["system"] = json!(system_prompt);
        }

        if let Some(ref re) = self.reasoning_effort {
            if re != "off" && self.model.contains("claude-3-7") {
                let budget = match re.as_str() {
                    "low" => 1024,
                    "medium" => 4096,
                    _ => 8192,
                };
                body["thinking"] = json!({ "type": "enabled", "budget_tokens": budget });
            }
        }

        if !tools.is_empty() {
            let tools_json: Vec<Value> = tools
                .into_iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "input_schema": t.parameters
                    })
                })
                .collect();
            body["tools"] = json!(tools_json);
        }

        let resp = self
            .client
            .post(url)
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&body)
            .send()
            .await
            .context("Failed to dispatch Anthropic request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("Anthropic error {}: {}", status, text);
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

        let mut current_tool_id: Option<String> = None;
        let mut current_tool_name: Option<String> = None;
        let mut current_block_index: usize = 0;

        while let Some(item) = stream.next().await {
            let bytes = item?;
            buffer.push_str(&String::from_utf8_lossy(&bytes));

            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer.drain(..pos + 1);

                if line.is_empty() || line.starts_with(':') {
                    continue;
                }

                if let Some(data) = line.strip_prefix("data: ")
                    && let Ok(val) = serde_json::from_str::<Value>(data) {
                        let event_type = val.get("type").and_then(|t| t.as_str()).unwrap_or("");

                        match event_type {
                            "content_block_start" => {
                                current_block_index = val
                                    .get("index")
                                    .and_then(|i| i.as_u64())
                                    .unwrap_or(0) as usize;
                                if let Some(cb) = val.get("content_block") {
                                    let cb_type =
                                        cb.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                    if cb_type == "tool_use" {
                                        current_tool_id = cb
                                            .get("id")
                                            .and_then(|i| i.as_str())
                                            .map(|s| s.to_string());
                                        current_tool_name = cb
                                            .get("name")
                                            .and_then(|n| n.as_str())
                                            .map(|s| s.to_string());
                                        let _ = tx
                                            .send(StreamChunk::ToolCallDelta {
                                                index: current_block_index,
                                                id: current_tool_id.clone(),
                                                name: current_tool_name.clone(),
                                                arguments_delta: String::new(),
                                            })
                                            .await;
                                    }
                                }
                            }
                            "content_block_delta" => {
                                if let Some(delta) = val.get("delta") {
                                    let d_type =
                                        delta.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                    if d_type == "text_delta" {
                                        if let Some(text) =
                                            delta.get("text").and_then(|t| t.as_str())
                                        {
                                            let _ = tx.send(StreamChunk::Token(text.to_string())).await;
                                        }
                                    } else if d_type == "thinking_delta" {
                                        if let Some(th) =
                                            delta.get("thinking").and_then(|t| t.as_str())
                                        {
                                            let _ = tx.send(StreamChunk::Thought(th.to_string())).await;
                                        }
                                    } else if d_type == "input_json_delta"
                                        && let Some(partial_json) =
                                            delta.get("partial_json").and_then(|p| p.as_str())
                                        {
                                            let _ = tx
                                                .send(StreamChunk::ToolCallDelta {
                                                    index: current_block_index,
                                                    id: current_tool_id.clone(),
                                                    name: current_tool_name.clone(),
                                                    arguments_delta: partial_json.to_string(),
                                                })
                                                .await;
                                        }
                                }
                            }
                            "message_stop" => {
                                let _ = tx.send(StreamChunk::Done).await;
                                return Ok(());
                            }
                            _ => {}
                        }
                    }
            }
        }

        let _ = tx.send(StreamChunk::Done).await;
        Ok(())
    }
}
