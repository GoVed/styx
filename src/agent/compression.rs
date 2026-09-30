use crate::router::openai::ChatMessageParam;
use serde_json::Value;

/// Conservative token estimator (chars / 3 or words * 4/3 + message boundary overhead)
pub fn estimate_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    let chars = text.chars().count();
    let words = text.split_whitespace().count();
    let est = (chars / 3).max((words * 4) / 3) + 4;
    est.max(1)
}

pub fn estimate_messages_tokens(messages: &[ChatMessageParam]) -> usize {
    messages
        .iter()
        .map(|m| {
            let mut est = estimate_tokens(&m.content);
            if let Some(imgs) = &m.images {
                est += imgs.len() * 500;
            }
            if let Some(tcs) = &m.tool_calls {
                for tc in tcs {
                    est += estimate_tokens(&tc.function.name)
                        + estimate_tokens(&tc.function.arguments)
                        + 8;
                }
            }
            est
        })
        .sum()
}

/// Recursively sanitizes JSON values to collapse massive base64 image strings or data URIs
fn sanitize_json_value(val: &mut Value) {
    match val {
        Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                let k_lower = k.to_lowercase();
                if (k_lower.contains("base64") || k_lower == "data" || k_lower == "buffer")
                    && let Some(s) = v.as_str()
                        && s.len() > 100 {
                            *v = Value::String(format!(
                                "[binary base64 payload omitted: {} chars]",
                                s.len()
                            ));
                            continue;
                        }
                sanitize_json_value(v);
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                sanitize_json_value(item);
            }
        }
        Value::String(s) => {
            if s.starts_with("data:image/") && s.contains(";base64,") && s.len() > 150 {
                *s = format!("[data-uri image omitted: {} chars]", s.len());
            }
        }
        _ => {}
    }
}

/// Strips giant binary base64 blobs and caps massive tool dumps to preserve model context
pub fn sanitize_message_content(content: &str, role: &str) -> String {
    if content.len() < 256 {
        return content.to_string();
    }

    // 1. If valid JSON (typical for tool outputs), collapse base64 fields
    if let Ok(mut val) = serde_json::from_str::<Value>(content) {
        sanitize_json_value(&mut val);
        let serialized = val.to_string();
        if role == "tool" && serialized.len() > 16384 {
            let prefix: String = serialized.chars().take(16384).collect();
            return format!(
                "{}\n\n[... Tool output truncated: showing first 16,384 of {} characters ...]",
                prefix,
                serialized.len()
            );
        }
        return serialized;
    }

    // 2. Cap raw text tool outputs at 16,384 chars
    if role == "tool" && content.len() > 16384 {
        let prefix: String = content.chars().take(16384).collect();
        return format!(
            "{}\n\n[... Tool output truncated: showing first 16,384 of {} characters ...]",
            prefix,
            content.len()
        );
    }

    content.to_string()
}

/// Auto-compresses message history to fit strictly within `max_prompt_budget` while preserving
/// the system prompt and the most recent conversation context.
pub fn prepare_auto_compressed_messages(
    messages: Vec<ChatMessageParam>,
    max_prompt_budget: usize,
) -> Vec<ChatMessageParam> {
    if messages.is_empty() {
        return messages;
    }

    // 1. Sanitize all messages to eliminate base64 blobs and cap tool dumps
    let sanitized: Vec<ChatMessageParam> = messages
        .into_iter()
        .map(|mut m| {
            m.content = sanitize_message_content(&m.content, &m.role);
            m
        })
        .collect();

    if estimate_messages_tokens(&sanitized) <= max_prompt_budget {
        return sanitized;
    }

    // Separate system message(s) from conversation history
    let (system_messages, mut conv_messages): (Vec<_>, Vec<_>) =
        sanitized.into_iter().partition(|m| m.role == "system");

    // Preserve the last 4 turns verbatim if possible
    let keep_verbatim_count = 4.min(conv_messages.len());
    let split_idx = conv_messages.len().saturating_sub(keep_verbatim_count);

    let older_messages = conv_messages.drain(..split_idx).collect::<Vec<_>>();
    let recent_messages = conv_messages;

    let mut result = system_messages;

    if !older_messages.is_empty() {
        // Compact older messages into a summary block
        let mut summary_lines = Vec::new();
        summary_lines.push("[Prior Conversation Context (Auto-Compressed Summary)]:".to_string());
        for msg in &older_messages {
            let role_label = match msg.role.as_str() {
                "user" => "User",
                "assistant" => "Styx",
                "tool" => msg.name.as_deref().unwrap_or("Tool Output"),
                r => r,
            };
            let cleaned = msg.content.replace('\n', " ").trim().to_string();
            let snippet = if cleaned.chars().count() > 240 {
                let truncated: String = cleaned.chars().take(240).collect();
                format!("{}...", truncated)
            } else {
                cleaned
            };
            if !snippet.is_empty() {
                summary_lines.push(format!("- {}: {}", role_label, snippet));
            }
        }
        summary_lines.push("[End of prior context]".to_string());

        let summary_msg = ChatMessageParam {
            role: "system".to_string(),
            content: summary_lines.join("\n"),
            name: None,
            tool_call_id: None,
            tool_calls: None,
            images: None,
        };
        result.push(summary_msg);
    }

    result.extend(recent_messages);

    // Progressive trim of recent turns if still over budget
    while estimate_messages_tokens(&result) > max_prompt_budget && result.len() > 3 {
        // Remove the oldest non-system message
        result.remove(2);
    }

    // HARD INVARIANT: Guarantee prompt fits strictly within max_prompt_budget
    while estimate_messages_tokens(&result) > max_prompt_budget {
        let current_tokens = estimate_messages_tokens(&result);
        if current_tokens <= max_prompt_budget {
            break;
        }

        // Find the index of the message with the highest token count (prefer turns at index >= 2)
        let candidate_idx = if result.len() > 2 {
            result
                .iter()
                .enumerate()
                .skip(2)
                .max_by_key(|(_, m)| m.content.len())
                .map(|(idx, _)| idx)
                .unwrap_or(1)
        } else if result.len() > 1 {
            1
        } else {
            0
        };

        let excess_tokens = current_tokens.saturating_sub(max_prompt_budget) + 64;
        let cut_chars = excess_tokens * 3;
        let msg_len = result[candidate_idx].content.len();

        if msg_len > cut_chars + 120 {
            let keep_len = msg_len.saturating_sub(cut_chars);
            let truncated: String = result[candidate_idx].content.chars().take(keep_len).collect();
            result[candidate_idx].content = format!(
                "{}\n\n[... Truncated by Styx Context Compressor to fit model window ...]",
                truncated
            );
        } else if result.len() > 3 && candidate_idx != 0 && candidate_idx != 1 && candidate_idx < result.len() - 1 {
            result.remove(candidate_idx);
        } else {
            let keep_len = msg_len.saturating_sub(cut_chars).max(80);
            result[candidate_idx].content =
                result[candidate_idx].content.chars().take(keep_len).collect();
            break;
        }
    }

    result
}
