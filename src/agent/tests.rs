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

#[test]
fn test_format_diagnosis_unknown_architecture() {
    let log = "llama_model_load: error loading model: unknown model architecture: 'k2-horizon'";
    let diag = format_diagnosis_from_logs(
        "IFM/K2-Horizon-7B-GGUF",
        131072,
        log,
        false,
        "exited (1)",
        log,
    );
    assert!(diag.contains("Unsupported Model Architecture in llama.cpp"));
    assert!(diag.contains("Qwen 3.5 9B (128k Local GGUF)"));
}

#[test]
fn test_find_repetition_cutoff_detects_repeated_phrase() {
    use super::reasoning::{find_repetition_cutoff, arrest_repetition};

    let normal_text = "Hello there! How can I help you today? Let me know if you need anything.";
    assert_eq!(find_repetition_cutoff(normal_text), None);

    let repeated = "Let me try with the correct source language setting.\n\nLet me try with the correct source language setting.\n\nLet me try with the correct source language setting.\n\n";
    let cutoff = find_repetition_cutoff(repeated);
    assert!(cutoff.is_some());
    let cutoff_idx = cutoff.unwrap();
    let truncated = &repeated[..cutoff_idx];
    assert!(truncated.contains("Let me try with the correct source language setting."));
    assert_eq!(truncated.matches("Let me try with the correct source language setting.").count(), 1);

    let mut current = repeated.to_string();
    let mut accumulated = repeated.to_string();
    let arrested = arrest_repetition(&mut current, &mut accumulated);
    assert!(arrested);
    assert_eq!(current.matches("Let me try with the correct source language setting.").count(), 1);
    assert_eq!(accumulated.matches("Let me try with the correct source language setting.").count(), 1);
}

#[test]
fn test_queue_priority_and_pruning_logic() {
    use std::collections::VecDeque;
    use chrono::{Utc, Duration};

    struct TestJob {
        id: &'static str,
        source: &'static str,
        session_id: &'static str,
        queued_at: chrono::DateTime<Utc>,
    }

    let now = Utc::now();
    let mut queue: VecDeque<TestJob> = VecDeque::new();

    // 1. Add background tool jobs
    queue.push_back(TestJob { id: "bg1", source: "tool_whatsapp", session_id: "s1", queued_at: now - Duration::seconds(400) });
    queue.push_back(TestJob { id: "bg2", source: "tool_whatsapp", session_id: "s1", queued_at: now - Duration::seconds(100) });

    // 2. Prune stale tool jobs (> 300s)
    queue.retain(|j| !(j.source.starts_with("tool_") && (now - j.queued_at).num_seconds() > 300));
    assert_eq!(queue.len(), 1);
    assert_eq!(queue[0].id, "bg2");

    // 3. Debounce: if >= 2 queued tool jobs for same session, reject 3rd
    queue.push_back(TestJob { id: "bg3", source: "tool_whatsapp", session_id: "s1", queued_at: now });
    let count_s1 = queue.iter().filter(|j| j.session_id == "s1" && j.source.starts_with("tool_")).count();
    assert_eq!(count_s1, 2);
    // 3rd trigger for s1 would be debounced
    let should_debounce = count_s1 >= 2;
    assert!(should_debounce);

    // 4. User priority insertion: user prompt must jump ahead of background tool jobs
    let user_job = TestJob { id: "user1", source: "user", session_id: "s1", queued_at: now };
    let insert_idx = queue.iter().position(|j| j.source != "user").unwrap_or(queue.len());
    assert_eq!(insert_idx, 0); // Jumps to index 0 ahead of bg2 and bg3!
    queue.insert(insert_idx, user_job);
    assert_eq!(queue[0].id, "user1");
    assert_eq!(queue[1].id, "bg2");
    assert_eq!(queue[2].id, "bg3");

    // Second user prompt inserts after user1 but before background jobs
    let user_job2 = TestJob { id: "user2", source: "user", session_id: "s2", queued_at: now };
    let insert_idx2 = queue.iter().position(|j| j.source != "user").unwrap_or(queue.len());
    assert_eq!(insert_idx2, 1);
    queue.insert(insert_idx2, user_job2);
    assert_eq!(queue[0].id, "user1");
    assert_eq!(queue[1].id, "user2");
    assert_eq!(queue[2].id, "bg2");
    assert_eq!(queue[3].id, "bg3");
}

#[test]
fn test_is_main_model_vision_capable_detection() {
    let mut cfg = crate::db::ModelConfigRecord {
        id: "test".to_string(),
        name: "Local llama.cpp (qwen-3-5-9b)".to_string(),
        provider: "docker_llamacpp".to_string(),
        base_url: Some("http://localhost:8080/v1".to_string()),
        api_key: None,
        model_id: "/models/Qwen3.5-9B-UD-Q4_K_XL.gguf".to_string(),
        context_length: 131072,
        is_active: true,
        created_at: "".to_string(),
        extra_flags_json: None,
    };
    assert!(super::tools::is_main_model_vision_capable(&cfg));

    cfg.model_id = "meta-llama/Llama-3.1-8B-Instruct".to_string();
    cfg.name = "Llama 3.1 8B".to_string();
    assert!(!super::tools::is_main_model_vision_capable(&cfg));

    cfg.provider = "anthropic".to_string();
    cfg.model_id = "claude-3-7-sonnet-20250219".to_string();
    assert!(super::tools::is_main_model_vision_capable(&cfg));
}

