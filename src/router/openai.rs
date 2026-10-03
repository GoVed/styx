pub use super::types::{ChatMessageParam, HandshakeResult, ToolCallFunctionParam, ToolCallParam, ToolParam};
use super::StreamTagParser;
use anyhow::{bail, Result};
use futures::StreamExt;
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Instant;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct OpenAiClient {
    client: Client,
    base_url: String,
    api_key: String,
    model: String,
    reasoning_effort: Option<String>,
}

impl OpenAiClient {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        Self::new_with_reasoning(base_url, api_key, model, None)
    }

    pub fn new_with_reasoning(
        base_url: String,
        api_key: String,
        model: String,
        reasoning_effort: Option<String>,
    ) -> Self {
        let mut clean_base = base_url.trim_end_matches('/').to_string();

        // If running inside Docker, adapt localhost / 127.0.0.1 to host.docker.internal
        // so containerized Styx can reach host-published ports and sibling model containers
        if std::path::Path::new("/.dockerenv").exists() {
            if clean_base.starts_with("http://localhost:") {
                clean_base = clean_base.replace("http://localhost:", "http://host.docker.internal:");
            } else if clean_base.starts_with("http://127.0.0.1:") {
                clean_base = clean_base.replace("http://127.0.0.1:", "http://host.docker.internal:");
            }
        }

        Self {
            client: Client::builder().build().unwrap_or_default(),
            base_url: clean_base,
            api_key,
            model,
            reasoning_effort,
        }
    }

    pub async fn test_connection(&self) -> HandshakeResult {
        let start = Instant::now();
        let models_url = format!("{}/models", self.base_url);

        let mut req = self.client.get(&models_url);
        if !self.api_key.is_empty() {
            req = req.bearer_auth(&self.api_key);
        }

        match req.send().await {
            Ok(resp) if resp.status().is_success() => {
                let latency_ms = start.elapsed().as_millis() as u64;
                let body: Value = resp.json().await.unwrap_or(Value::Null);
                let mut models = Vec::new();
                if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
                    for m in data {
                        if let Some(id) = m.get("id").and_then(|i| i.as_str()) {
                            models.push(id.to_string());
                        }
                    }
                }
                HandshakeResult {
                    success: true,
                    message: format!("Successfully reached endpoint at {} ({} models found)", self.base_url, models.len()),
                    latency_ms,
                    models,
                }
            }
            _ => {
                // Try 1-token completion test
                let chat_url = format!("{}/chat/completions", self.base_url);
                let payload = json!({
                    "model": self.model,
                    "messages": [{"role": "user", "content": "ping"}],
                    "max_tokens": 1
                });
                let mut req2 = self.client.post(&chat_url).json(&payload);
                if !self.api_key.is_empty() {
                    req2 = req2.bearer_auth(&self.api_key);
                }

                match req2.send().await {
                    Ok(resp2) if resp2.status().is_success() => {
                        let latency_ms = start.elapsed().as_millis() as u64;
                        HandshakeResult {
                            success: true,
                            message: "Endpoint responded to ping chat completion successfully!".to_string(),
                            latency_ms,
                            models: vec![self.model.clone()],
                        }
                    }
                    Ok(resp2) => {
                        let status = resp2.status();
                        let text = resp2.text().await.unwrap_or_default();
                        HandshakeResult {
                            success: false,
                            message: format!("HTTP error {}: {}", status, text),
                            latency_ms: start.elapsed().as_millis() as u64,
                            models: Vec::new(),
                        }
                    }
                    Err(e) => {
                        HandshakeResult {
                            success: false,
                            message: format!("Connection failed: {}", e),
                            latency_ms: start.elapsed().as_millis() as u64,
                            models: Vec::new(),
                        }
                    }
                }
            }
        }
    }

    pub async fn stream_chat(
        &self,
        messages: Vec<ChatMessageParam>,
        tools: Vec<ToolParam>,
        tx: mpsc::Sender<super::StreamChunk>,
        max_tokens: Option<u32>,
    ) -> Result<()> {
        let chat_url = format!("{}/chat/completions", self.base_url);
        let prepared_messages = super::vision::prepare_openai_messages(&messages).await;

        let mut body = json!({
            "model": self.model,
            "messages": prepared_messages,
            "stream": true,
            "temperature": 0.7,
            "frequency_penalty": 0.2,
            "presence_penalty": 0.2,
        });

        if let Some(mt) = max_tokens {
            body["max_tokens"] = json!(mt);
        }

        if let Some(ref re) = self.reasoning_effort {
            if re != "off" {
                body["reasoning_effort"] = json!(re);
            }
        }

        if !tools.is_empty() {
            let tools_json: Vec<Value> = tools
                .into_iter()
                .map(|t| {
                    json!({
                        "type": "function",
                        "function": {
                            "name": t.name,
                            "description": t.description,
                            "parameters": t.parameters
                        }
                    })
                })
                .collect();
            body["tools"] = json!(tools_json);
        }

        let mut req = self.client.post(&chat_url).json(&body);
        if !self.api_key.is_empty() {
            req = req.bearer_auth(&self.api_key);
        }

        let mut resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                let err_str = e.to_string();
                if e.is_connect() || err_str.contains("reset by peer") || err_str.contains("refused") {
                    bail!(
                        "Inference engine at {} is warming up or not reachable yet. If the container was just deployed, model compilation/loading is in progress. Please check the container logs in Model Orchestrator.",
                        chat_url
                    );
                }
                bail!("Inference request failed for {}: {}", chat_url, e);
            }
        };

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();

            // 1. If failure was due to image input not supported on a text-only model (missing mmproj), perceive or strip images
            if text.contains("image input is not supported") || text.contains("mmproj") {
                let raw_msgs = body.get("messages").and_then(|m| m.as_array()).map(|a| a.as_slice()).unwrap_or(&[]);
                let perceived_messages = super::perceiver::perceive_or_strip_images(raw_msgs, None).await;
                let mut retry_body = body.clone();
                retry_body["messages"] = json!(perceived_messages);
                let mut retry_req = self.client.post(&chat_url).json(&retry_body);
                if !self.api_key.is_empty() {
                    retry_req = retry_req.bearer_auth(&self.api_key);
                }
                let retry_resp = retry_req.send().await.map_err(|e| anyhow::anyhow!("Retry without images failed: {}", e))?;
                if retry_resp.status().is_success() {
                    resp = retry_resp;
                } else {
                    let r_status = retry_resp.status();
                    let r_text = retry_resp.text().await.unwrap_or_default();
                    bail!("OpenAI API error {}: {}", r_status, r_text);
                }
            } else if body.get("tools").is_some() && (text.contains("tool choice") || text.contains("tool") || text.contains("function")) {
                let mut retry_body = body.clone();
                if let Some(obj) = retry_body.as_object_mut() {
                    obj.remove("tools");
                }
                let mut retry_req = self.client.post(&chat_url).json(&retry_body);
                if !self.api_key.is_empty() {
                    retry_req = retry_req.bearer_auth(&self.api_key);
                }
                let retry_resp = retry_req.send().await.map_err(|e| anyhow::anyhow!("Retry without tools failed: {}", e))?;
                if retry_resp.status().is_success() {
                    resp = retry_resp;
                } else {
                    let r_status = retry_resp.status();
                    let r_text = retry_resp.text().await.unwrap_or_default();
                    bail!("OpenAI API error {}: {}", r_status, r_text);
                }
            } else if body.get("reasoning_effort").is_some() && (text.contains("reasoning_effort") || text.contains("extra_forbidden") || text.contains("unrecognized")) {
                let mut retry_body = body.clone();
                if let Some(obj) = retry_body.as_object_mut() {
                    obj.remove("reasoning_effort");
                }
                let mut retry_req = self.client.post(&chat_url).json(&retry_body);
                if !self.api_key.is_empty() {
                    retry_req = retry_req.bearer_auth(&self.api_key);
                }
                let retry_resp = retry_req.send().await.map_err(|e| anyhow::anyhow!("Retry without reasoning_effort failed: {}", e))?;
                if retry_resp.status().is_success() {
                    resp = retry_resp;
                } else {
                    let r_status = retry_resp.status();
                    let r_text = retry_resp.text().await.unwrap_or_default();
                    bail!("OpenAI API error {}: {}", r_status, r_text);
                }
            } else {
                bail!("OpenAI API error {}: {}", status, text);
            }
        }

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();
        let mut tag_parser = StreamTagParser::new();

        while let Some(item) = stream.next().await {
            let bytes = item?;
            buffer.push_str(&String::from_utf8_lossy(&bytes));

            while let Some(pos) = buffer.find('\n') {
                let line = buffer[..pos].trim().to_string();
                buffer.drain(..pos + 1);

                if line.is_empty() || line.starts_with(':') {
                    continue;
                }

                if line == "data: [DONE]" {
                    for chunk in tag_parser.flush() {
                        let _ = tx.send(chunk).await;
                    }
                    let _ = tx.send(super::StreamChunk::Done).await;
                    return Ok(());
                }

                if let Some(data) = line.strip_prefix("data: ")
                    && let Ok(val) = serde_json::from_str::<Value>(data)
                        && let Some(choices) = val.get("choices").and_then(|c| c.as_array())
                            && let Some(first) = choices.first() {
                                let delta = first.get("delta");

                                // 1. Check reasoning / thought tokens (e.g. DeepSeek / Qwen / vLLM reasoning_content)
                                if let Some(thought) = delta
                                    .and_then(|d| d.get("reasoning_content"))
                                    .and_then(|r| r.as_str())
                                    && !thought.is_empty() {
                                        let _ = tx.send(super::StreamChunk::Thought(thought.to_string())).await;
                                    }

                                // 2. Check content tokens (parsed for thoughts and native XML/JSON tool calls)
                                if let Some(content) = delta
                                    .and_then(|d| d.get("content"))
                                    .and_then(|c| c.as_str())
                                    && !content.is_empty() {
                                        for chunk in tag_parser.process(content) {
                                            let _ = tx.send(chunk).await;
                                        }
                                    }

                                // 3. Check tool calls delta
                                if let Some(tool_calls) = delta
                                    .and_then(|d| d.get("tool_calls"))
                                    .and_then(|tc| tc.as_array())
                                {
                                    for tc in tool_calls {
                                        let index = tc.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                                        let id = tc.get("id").and_then(|i| i.as_str()).map(|s| s.to_string());
                                        let name = tc
                                            .get("function")
                                             .and_then(|f| f.get("name"))
                                            .and_then(|n| n.as_str())
                                            .map(|s| s.to_string());
                                        let args = tc
                                            .get("function")
                                            .and_then(|f| f.get("arguments"))
                                            .and_then(|a| a.as_str())
                                            .unwrap_or("")
                                            .to_string();

                                        let _ = tx
                                            .send(super::StreamChunk::ToolCallDelta {
                                                index,
                                                id,
                                                name,
                                                arguments_delta: args,
                                            })
                                            .await;
                                    }
                                }
                            }
            }
        }

        for chunk in tag_parser.flush() {
            let _ = tx.send(chunk).await;
        }
        let _ = tx.send(super::StreamChunk::Done).await;
        Ok(())
    }
}
