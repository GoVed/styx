use anyhow::{bail, Context, Result};
use futures::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Instant;
use tokio::sync::mpsc;

use super::openai::{ChatMessageParam, HandshakeResult, ToolParam};
use super::StreamChunk;

#[derive(Clone)]
pub struct GeminiClient {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiClient {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::builder().build().unwrap_or_default(),
            api_key,
            model,
        }
    }

    pub async fn test_connection(&self) -> HandshakeResult {
        let start = Instant::now();
        if self.api_key.is_empty() {
            return HandshakeResult {
                success: false,
                message: "Gemini API key is empty".to_string(),
                latency_ms: 0,
                models: Vec::new(),
            };
        }

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let body = json!({
            "contents": [{"parts": [{"text": "ping"}]}],
            "generationConfig": {"maxOutputTokens": 1}
        });

        match self.client.post(&url).json(&body).send().await {
            Ok(resp) if resp.status().is_success() => {
                let latency_ms = start.elapsed().as_millis() as u64;
                HandshakeResult {
                    success: true,
                    message: format!("Gemini connection verified with model {}", self.model),
                    latency_ms,
                    models: vec![self.model.clone()],
                }
            }
            Ok(resp) => {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                HandshakeResult {
                    success: false,
                    message: format!("Gemini API error {}: {}", status, text),
                    latency_ms: start.elapsed().as_millis() as u64,
                    models: Vec::new(),
                }
            }
            Err(e) => HandshakeResult {
                success: false,
                message: format!("Gemini connection failed: {}", e),
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
        tool_choice: Option<String>,
    ) -> Result<()> {
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:streamGenerateContent?key={}&alt=sse",
            self.model, self.api_key
        );

        let mut contents = Vec::new();
        let mut system_instruction = None;

        for m in messages {
            if m.role == "system" {
                system_instruction = Some(json!({
                    "parts": [{"text": m.content}]
                }));
            } else if m.role == "tool" {
                contents.push(json!({
                    "role": "function",
                    "parts": [{
                        "functionResponse": {
                            "name": m.name.unwrap_or_else(|| "tool".to_string()),
                            "response": { "output": m.content }
                        }
                    }]
                }));
            } else if m.role == "assistant" && m.tool_calls.is_some() {
                let mut parts = Vec::new();
                if !m.content.trim().is_empty() {
                    parts.push(json!({ "text": m.content }));
                }
                for tc in m.tool_calls.unwrap_or_default() {
                    let args: Value = serde_json::from_str(&tc.function.arguments).unwrap_or(json!({}));
                    parts.push(json!({
                        "functionCall": {
                            "name": tc.function.name,
                            "args": args
                        }
                    }));
                }
                contents.push(json!({
                    "role": "model",
                    "parts": parts
                }));
            } else {
                let role = if m.role == "user" { "user" } else { "model" };
                contents.push(json!({
                    "role": role,
                    "parts": [{"text": m.content}]
                }));
            }
        }

        let mut body = json!({
            "contents": contents,
        });

        if let Some(mt) = max_tokens {
            body["generationConfig"] = json!({
                "maxOutputTokens": mt.min(8192)
            });
        }

        if let Some(sys) = system_instruction {
            body["systemInstruction"] = sys;
        }

        if !tools.is_empty() {
            let func_decls: Vec<Value> = tools
                .into_iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "parameters": t.parameters
                    })
                })
                .collect();
            body["tools"] = json!([{
                "functionDeclarations": func_decls
            }]);

            if let Some(ref tc) = tool_choice {
                if tc == "required" {
                    body["toolConfig"] = json!({
                        "functionCallingConfig": { "mode": "ANY" }
                    });
                }
            }
        }

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to dispatch Gemini request")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            bail!("Gemini error {}: {}", status, text);
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();

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
                    && let Ok(val) = serde_json::from_str::<Value>(data)
                        && let Some(candidates) = val.get("candidates").and_then(|c| c.as_array())
                            && let Some(first) = candidates.first()
                                && let Some(content) = first.get("content")
                                    && let Some(parts) =
                                        content.get("parts").and_then(|p| p.as_array())
                                    {
                                        for (idx, part) in parts.iter().enumerate() {
                                            if let Some(text) =
                                                part.get("text").and_then(|t| t.as_str())
                                            {
                                                let _ = tx
                                                    .send(StreamChunk::Token(text.to_string()))
                                                    .await;
                                            } else if let Some(fc) = part.get("functionCall") {
                                                let name = fc
                                                    .get("name")
                                                    .and_then(|n| n.as_str())
                                                    .unwrap_or_default()
                                                    .to_string();
                                                let args = fc
                                                    .get("args")
                                                    .cloned()
                                                    .unwrap_or(Value::Null)
                                                    .to_string();

                                                let _ = tx
                                                    .send(StreamChunk::ToolCallDelta {
                                                        index: idx,
                                                        id: Some(format!("call_{}", idx)),
                                                        name: Some(name),
                                                        arguments_delta: args,
                                                    })
                                                    .await;
                                            }
                                        }
                                    }
            }
        }

        let _ = tx.send(StreamChunk::Done).await;
        Ok(())
    }
}
