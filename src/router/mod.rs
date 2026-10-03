pub mod anthropic;
pub mod gemini;
pub mod openai;
pub mod perceiver;
pub mod streaming;
pub mod tags;
pub mod types;
pub mod vision;

pub use streaming::{StreamChunk, StreamTagParser};
pub use types::{ChatMessageParam, HandshakeResult, ToolParam};

use anthropic::AnthropicClient;
use gemini::GeminiClient;
use openai::OpenAiClient;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct MultiModelRouter {
    // Shared router
}

impl MultiModelRouter {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn test_provider(
        &self,
        provider: &str,
        base_url: Option<&str>,
        api_key: Option<&str>,
        model: &str,
    ) -> HandshakeResult {
        match provider {
            "anthropic" => {
                let client = AnthropicClient::new(
                    api_key.unwrap_or_default().to_string(),
                    model.to_string(),
                );
                client.test_connection().await
            }
            "gemini" => {
                let client = GeminiClient::new(
                    api_key.unwrap_or_default().to_string(),
                    model.to_string(),
                );
                client.test_connection().await
            }
            _ => {
                // OpenAI compatible (vLLM, llama.cpp, Ollama, OpenAI)
                let base = base_url.unwrap_or("http://localhost:8000/v1");
                let client = OpenAiClient::new(
                    base.to_string(),
                    api_key.unwrap_or_default().to_string(),
                    model.to_string(),
                );
                client.test_connection().await
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn dispatch_stream(
        &self,
        provider: &str,
        base_url: Option<&str>,
        api_key: Option<&str>,
        model: &str,
        messages: Vec<ChatMessageParam>,
        tools: Vec<ToolParam>,
        max_tokens: Option<u32>,
        reasoning_effort: Option<&str>,
        tool_choice: Option<&str>,
    ) -> mpsc::Receiver<StreamChunk> {
        let (tx, rx) = mpsc::channel(100);

        let provider_str = provider.to_string();
        let base_url_str = base_url.map(|s| s.to_string());
        let api_key_str = api_key.unwrap_or_default().to_string();
        let model_str = model.to_string();
        let reasoning_str = reasoning_effort.map(|s| s.to_string());
        let tool_choice_str = tool_choice.map(|s| s.to_string());

        tokio::spawn(async move {
            let res = match provider_str.as_str() {
                "anthropic" => {
                    let client = AnthropicClient::new_with_reasoning(api_key_str, model_str, reasoning_str);
                    client.stream_chat(messages, tools, tx.clone(), max_tokens, tool_choice_str).await
                }
                "gemini" => {
                    let client = GeminiClient::new(api_key_str, model_str);
                    client.stream_chat(messages, tools, tx.clone(), max_tokens, tool_choice_str).await
                }
                _ => {
                    let base = base_url_str.unwrap_or_else(|| "http://localhost:8000/v1".to_string());
                    let client = OpenAiClient::new_with_reasoning(base, api_key_str, model_str, reasoning_str);
                    client.stream_chat(messages, tools, tx.clone(), max_tokens, tool_choice_str).await
                }
            };

            if let Err(e) = res {
                let err_msg = format!("{:?}", e);
                // If endpoint cannot be connected (e.g. Docker container still pulling or offline),
                // provide helpful fallback instructions.
                if err_msg.contains("Connection refused")
                    || err_msg.contains("dns error")
                    || err_msg.contains("ConnectError")
                {
                    let _ = tx.send(StreamChunk::Thought(
                        "Connecting to local inference engine...\nTarget endpoint is currently offline or still starting up.".to_string()
                    )).await;
                    let _ = tx.send(StreamChunk::Token(format!(
                        "### ⚠️ Local Model Endpoint Offline\n\nCould not connect to model endpoint (`{}`).\n\n**To resolve:**\n1. Go to the **Model Management** tab in the top navigation.\n2. Spin up a container (vLLM, llama.cpp, or Ollama) using the provisioner, OR configure an external API key (Anthropic Claude, Google Gemini, or OpenAI).\n3. Click **Set as Active Model** and try again.",
                        provider_str
                    ))).await;
                    let _ = tx.send(StreamChunk::Done).await;
                } else {
                    let _ = tx.send(StreamChunk::Error(err_msg)).await;
                }
            }
        });

        rx
    }
}
