use super::compression::*;
use super::diagnosis::*;
use crate::router::openai::ChatMessageParam;

#[test]
fn test_estimate_tokens() {
    assert_eq!(estimate_tokens(""), 0);
    let t1 = estimate_tokens("Hello world");
    assert!(t1 >= 4);

    let long_text = "a".repeat(300);
    let t2 = estimate_tokens(&long_text);
    assert_eq!(t2, 104); // 300 / 3 + 4
}

#[test]
fn test_prepare_auto_compressed_messages_below_budget() {
    let msgs = vec![
        ChatMessageParam {
            role: "system".to_string(),
            content: "You are Styx.".to_string(),
            name: None,
            tool_call_id: None,
            tool_calls: None,
            images: None,
        },
        ChatMessageParam {
            role: "user".to_string(),
            content: "Hi".to_string(),
            name: None,
            tool_call_id: None,
            tool_calls: None,
            images: None,
        },
    ];
    let res = prepare_auto_compressed_messages(msgs.clone(), 1000);
    assert_eq!(res.len(), 2);
    assert_eq!(res[0].content, "You are Styx.");
    assert_eq!(res[1].content, "Hi");
}

#[test]
fn test_prepare_auto_compressed_messages_compresses_older_turns() {
    let mut msgs = vec![ChatMessageParam {
        role: "system".to_string(),
        content: "You are Styx.".to_string(),
        name: None,
        tool_call_id: None,
        tool_calls: None,
        images: None,
    }];

    // Add 10 turns
    for i in 0..10 {
        msgs.push(ChatMessageParam {
            role: if i % 2 == 0 {
                "user".to_string()
            } else {
                "assistant".to_string()
            },
            content: format!("This is turn number {} with some details and information.", i),
            name: None,
            tool_call_id: None,
            tool_calls: None,
            images: None,
        });
    }

    // Budget that forces compression
    let res = prepare_auto_compressed_messages(msgs, 100);
    // Should have system instruction + summary block + last 4 turns (or trimmed to budget)
    assert!(res
        .iter()
        .any(|m| m.content.contains("[Prior Conversation Context (Auto-Compressed Summary)]")));
    // The last turn must be preserved
    assert_eq!(
        res.last().unwrap().content,
        "This is turn number 9 with some details and information."
    );
}

#[test]
fn test_sanitize_message_content_collapses_base64() {
    let giant_b64 = "A".repeat(50000);
    let json_content = format!(r#"{{"success":true,"filename":"sticker.webp","base64":"{}"}}"#, giant_b64);
    let sanitized = sanitize_message_content(&json_content, "tool");
    assert!(!sanitized.contains(&giant_b64));
    assert!(sanitized.contains("[binary base64 payload omitted: 50000 chars]"));
}

#[test]
fn test_prepare_auto_compressed_messages_strictly_enforces_budget_on_giant_tool_message() {
    let giant_output = "x".repeat(300000); // 300,000 characters
    let msgs = vec![
        ChatMessageParam {
            role: "system".to_string(),
            content: "You are Styx.".to_string(),
            name: None,
            tool_call_id: None,
            tool_calls: None,
            images: None,
        },
        ChatMessageParam {
            role: "user".to_string(),
            content: "download a sticker".to_string(),
            name: None,
            tool_call_id: None,
            tool_calls: None,
            images: None,
        },
        ChatMessageParam {
            role: "tool".to_string(),
            content: giant_output,
            name: Some("download_sticker".to_string()),
            tool_call_id: Some("call_1".to_string()),
            tool_calls: None,
            images: None,
        },
    ];

    let max_budget = 600;
    let res = prepare_auto_compressed_messages(msgs, max_budget);
    let final_tokens = estimate_messages_tokens(&res);
    assert!(
        final_tokens <= max_budget,
        "Final tokens {} must be <= max budget {}",
        final_tokens,
        max_budget
    );
}

#[test]
fn test_format_diagnosis_out_of_memory() {
    let log = "(EngineCore pid=263) ValueError: No available memory for the cache blocks. Try increasing `gpu_memory_utilization`";
    let diag = format_diagnosis_from_logs(
        "QuantTrio/Qwen3.5-9B-AWQ",
        65536,
        log,
        false,
        "exited (1)",
        "",
    );
    assert!(diag.contains("Graphics Memory (VRAM) Limit Exceeded"));
    assert!(diag.contains("<option>Re-deploy with 16k context window (Recommended)</option>"));
    assert!(diag.contains("<option>Switch to Cloud AI (Claude / Gemini)</option>"));

    let log2 = "ValueError: To serve at least one request with the model's max seq len (131072), 1.45 GiB KV cache is needed, which is larger than the available KV cache memory (0.2 GiB). Try increasing `gpu_memory_utilization` or decreasing `max_model_len`";
    let diag2 = format_diagnosis_from_logs(
        "QuantTrio/Qwen3.5-9B-AWQ",
        131072,
        log2,
        false,
        "exited (1)",
        "",
    );
    assert!(diag2.contains("Graphics Memory (VRAM) Limit Exceeded"));
}

#[test]
fn test_format_diagnosis_warming_up() {
    let log = "(EngineCore pid=263) Loading safetensors checkpoint shards: 60% Completed";
    let diag = format_diagnosis_from_logs(
        "QuantTrio/Qwen3.5-9B-AWQ",
        16384,
        log,
        true,
        "running",
        "",
    );
    assert!(diag.contains("AI Engine is Initializing"));
    assert!(diag.contains("<option>Wait 20 seconds and retry automatically</option>"));
}

#[test]
fn test_format_diagnosis_container_stopped() {
    let diag = format_diagnosis_from_logs(
        "QuantTrio/Qwen3.5-9B-AWQ",
        16384,
        "Fatal error",
        false,
        "exited (137)",
        "Fatal error",
    );
    assert!(diag.contains("Model Engine Stopped"));
    assert!(diag.contains("<option>Restart model container</option>"));
}

#[test]
fn test_format_diagnosis_repo_not_found() {
    let log = "OSError: Qwen/Qwen3.8-27B-Instruct-AWQ is not a valid model identifier listed on 'https://huggingface.co/models'";
    let diag = format_diagnosis_from_logs(
        "Qwen/Qwen3.8-27B-Instruct-AWQ",
        65536,
        log,
        false,
        "exited (1)",
        log,
    );
    assert!(diag.contains("Model Repository Not Found on Hugging Face"));
    assert!(diag.contains("QuantTrio/Qwen3.5-9B-AWQ"));
    assert!(diag.contains("<option>Deploy verified 9B model (QuantTrio/Qwen3.5-9B-AWQ)</option>"));
}

#[test]
fn test_format_diagnosis_invalid_choice() {
    let log = "vllm serve: error: argument --kv-cache-dtype: invalid choice: 'q4'";
    let diag = format_diagnosis_from_logs(
        "QuantTrio/Qwen3.5-9B-AWQ",
        16384,
        log,
        false,
        "exited (2)",
        log,
    );
    assert!(diag.contains("Invalid Engine Configuration Argument"));
    assert!(diag.contains("<option>Re-deploy with recommended engine defaults</option>"));
}
