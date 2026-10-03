use std::time::Instant;

use crate::db::ModelConfigRecord;
use crate::docker::DockerOrchestrator;

pub async fn probe_engine_ready(base_url: Option<&str>, timeout_secs: u64) -> bool {
    let raw_url = base_url.unwrap_or("http://localhost:8000/v1");
    let mut probe_url = format!("{}/models", raw_url.trim_end_matches('/'));
    if std::path::Path::new("/.dockerenv").exists() {
        probe_url = probe_url
            .replace("http://localhost:", "http://host.docker.internal:")
            .replace("http://127.0.0.1:", "http://host.docker.internal:");
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(1500))
        .build()
        .unwrap_or_default();

    let start = Instant::now();
    while start.elapsed().as_secs() < timeout_secs {
        if let Ok(resp) = client.get(&probe_url).send().await
            && resp.status().is_success() {
                return true;
            }
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
    }
    false
}

pub async fn diagnose_inference_failure(
    docker: &DockerOrchestrator,
    model_cfg: &ModelConfigRecord,
    err_msg: &str,
) -> String {
    if err_msg.contains("exceeds the available context size")
        || err_msg.contains("exceed_context_size_error")
        || err_msg.contains("context_length_exceeded")
    {
        return format!(
            "⚠️ **Model Context Window Exceeded ({ctx} tokens)**\n\n\
            The conversation history and active tool definitions exceeded the engine's current allocation ({ctx} tokens).\n\n\
            **Why this happened:** The active container was launched with a `{ctx}` token context size (`-c {ctx}`). Extensive message histories or multi-turn tool dumps required more tokens than allocated.\n\n\
            **Recommended Solution:** Re-deploy or switch to **Qwen 3.5 9B with full 128k context window (131,072 tokens)**, which fits comfortably in GPU memory with Q8/Q4 KV cache.\n\n\
            <options>\n\
            <option>Deploy Qwen 3.5 9B (128k Local GGUF)</option>\n\
            <option>Re-deploy with 16k context window (Recommended)</option>\n\
            <option>Switch to Cloud AI</option>\n\
            </options>",
            ctx = model_cfg.context_length
        );
    }

    let is_docker = model_cfg.provider.starts_with("docker_")
        || model_cfg
            .base_url
            .as_deref()
            .unwrap_or("")
            .contains("8000")
        || model_cfg
            .base_url
            .as_deref()
            .unwrap_or("")
            .contains("localhost")
        || model_cfg
            .base_url
            .as_deref()
            .unwrap_or("")
            .contains("host.docker.internal");

    if is_docker
        && let Ok(containers) = docker.list_containers(true).await {
            let target = containers.into_iter().find(|c| {
                c.is_syndae_managed
                    || c.model_id.as_deref() == Some(&model_cfg.model_id)
                    || c.names
                        .iter()
                        .any(|n| n.contains("syndae") || n.contains("vllm") || n.contains("qwen"))
                    || c.ports
                        .iter()
                        .any(|p| p.contains("8000") || p.contains("8080") || p.contains("11434"))
            });

            if let Some(c) = target {
                let logs = docker.get_container_logs(&c.id, 40).await.unwrap_or_default();
                let log_str = logs.join("\n");
                let recent_preview = logs
                    .iter()
                    .rev()
                    .take(5)
                    .cloned()
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join("\n");

                return format_diagnosis_from_logs(
                    &model_cfg.model_id,
                    model_cfg.context_length,
                    &log_str,
                    c.state == "running",
                    &c.status,
                    &recent_preview,
                );
            } else {
                return "⚠️ **No Local AI Container Running**\n\n\
                    No inference engine was detected on port 8000.\n\n\
                    Would you like to deploy Qwen 3.5 9B with safe 16k settings or connect to an external provider?\n\n\
                    <options>\n\
                    <option>Re-deploy with 16k context window (Recommended)</option>\n\
                    <option>Switch to Cloud AI</option>\n\
                    <option>Open Model Manager</option>\n\
                    </options>".to_string();
            }
        }

    // Generic fallback
    format!(
        "⚠️ **Inference Engine Not Reachable**\n\n\
        Could not connect to `{url}`: {err}\n\n\
        The service may still be warming up or preparing its endpoint.\n\n\
        <options>\n\
        <option>Wait 20 seconds and retry automatically</option>\n\
        <option>Re-deploy with 16k context window (Recommended)</option>\n\
        <option>View recent error logs</option>\n\
        <option>Switch to Cloud AI</option>\n\
        </options>",
        url = model_cfg.base_url.as_deref().unwrap_or("local engine"),
        err = err_msg
    )
}

pub fn format_diagnosis_from_logs(
    model: &str,
    ctx: i64,
    log_str: &str,
    is_running: bool,
    status: &str,
    recent_preview: &str,
) -> String {
    // 1. Out of memory / cache block allocation failure
    if log_str.contains("No available memory for the cache blocks")
        || log_str.contains("OutOfMemoryError")
        || log_str.contains("out of memory")
        || log_str.contains("CUDA out of memory")
        || log_str.contains("HIP out of memory")
        || log_str.contains("larger than the available KV cache memory")
        || log_str.contains("To serve at least one request with the model's max seq len")
    {
        return format!(
            "⚠️ **Graphics Memory (VRAM) Limit Exceeded**\n\n\
            The model `{model}` could not fit the requested memory window ({ctx} tokens) into your graphics card (16 GB VRAM).\n\n\
            **Why this happened:** The model itself takes ~11.2 GB of graphics memory for its neural weights and vision processing. At higher context lengths, the temporary conversation memory (KV cache) exceeded the remaining ~3.5 GB of VRAM, causing the engine to stop.\n\n\
            **Recommended Fix:** A **16k context window** (16,384 tokens) fits comfortably in memory and runs fast (25–30+ words/sec) on this GPU.\n\n\
            <options>\n\
            <option>Re-deploy with 16k context window (Recommended)</option>\n\
            <option>Switch to Cloud AI (Claude / Gemini)</option>\n\
            <option>View recent error logs</option>\n\
            </options>"
        );
    }

    // 2. Warmup in progress
    if is_running
        && (log_str.contains("Loading safetensors")
            || log_str.contains("Loading model weights")
            || log_str.contains("Warmup run finished")
            || log_str.contains("Capturing CUDA graph")
            || log_str.contains("init engine")
            || log_str.contains("llama_model_load:"))
    {
        return format!(
            "⏳ **AI Engine is Initializing**\n\n\
            The model `{model}` is currently being loaded into graphics memory (VRAM) and warming up execution kernels.\n\n\
            Initial load typically takes 15–30 seconds for a 9B model. Please wait a brief moment.\n\n\
            <options>\n\
            <option>Wait 20 seconds and retry automatically</option>\n\
            <option>View recent error logs</option>\n\
            <option>Switch to Cloud AI</option>\n\
            </options>"
        );
    }

    // 3. Container stopped or crashed
    if !is_running {
        // 3a. Hugging Face repository or file not found
        if log_str.contains("is not a valid model identifier")
            || log_str.contains("RepositoryNotFoundError")
            || log_str.contains("404 Client Error")
            || log_str.contains("Entry Not Found")
        {
            return format!(
                "❌ **Model Repository Not Found on Hugging Face**\n\n\
                The specified model repository `{model}` does not exist or is private/inaccessible.\n\n\
                **Details:**\n```\n{preview}\n```\n\n\
                **Recommended Action:** Deploy the verified local QuantTrio 9B model or open Model Manager to re-configure.\n\n\
                <options>\n\
                <option>Deploy verified 9B model (QuantTrio/Qwen3.5-9B-AWQ)</option>\n\
                <option>Open Model Manager</option>\n\
                <option>Switch to Cloud AI</option>\n\
                </options>",
                preview = if recent_preview.is_empty() { "Repository 404 Not Found on Hugging Face" } else { recent_preview }
            );
        }

        // 3b. Unknown or unsupported model architecture in engine
        if log_str.contains("unknown model architecture") {
            return format!(
                "⚠️ **Unsupported Model Architecture in llama.cpp**\n\n\
                The architecture for model `{model}` is not yet supported in upstream mainline `llama.cpp`.\n\n\
                **Details:**\n```\n{preview}\n```\n\n\
                **Why this happened:** The K2-Horizon architecture was recently released (September 2026). Mainline llama.cpp has an open pull request but has not yet merged native `k2-horizon` support. The model authors currently maintain a dedicated fork (`MBZUAI-IFM/llama.cpp`).\n\n\
                **Recommended Action:** Switch to a fully supported 128k local model (like Qwen 3.5 9B with 128k context) or use Cloud AI.\n\n\
                <options>\n\
                <option>Deploy Qwen 3.5 9B (128k Local GGUF)</option>\n\
                <option>Open Model Manager</option>\n\
                <option>Switch to Cloud AI</option>\n\
                </options>",
                preview = if recent_preview.is_empty() { "unknown model architecture" } else { recent_preview }
            );
        }

        // 3c. CLI argument or engine parameter validation failure
        if log_str.contains("unrecognized arguments")
            || log_str.contains("invalid choice:")
            || log_str.contains("argument --")
            || log_str.contains("error: unrecognized")
        {
            return format!(
                "❌ **Invalid Engine Configuration Argument**\n\n\
                The inference engine rejected one or more command-line arguments.\n\n\
                **Details:**\n```\n{preview}\n```\n\n\
                **Recommended Action:** Re-deploy using verified default arguments or adjust options in Model Manager.\n\n\
                <options>\n\
                <option>Re-deploy with recommended engine defaults</option>\n\
                <option>Open Model Manager</option>\n\
                <option>Switch to Cloud AI</option>\n\
                </options>",
                preview = if recent_preview.is_empty() { "Invalid CLI arguments provided to engine" } else { recent_preview }
            );
        }

        return format!(
            "❌ **Model Engine Stopped**\n\n\
            The local AI container is not running (status: `{status}`).\n\n\
            **Recent Container Output:**\n```\n{preview}\n```\n\n\
            You can restart the container, deploy with a safe configuration, or switch to Cloud AI.\n\n\
            <options>\n\
            <option>Restart model container</option>\n\
            <option>Re-deploy with 16k context window (Recommended)</option>\n\
            <option>Switch to Cloud AI</option>\n\
            </options>",
            preview = if recent_preview.is_empty() { "No error logs captured" } else { recent_preview }
        );
    }

    // 4. Default running but slow/unresponsive
    "⏳ **AI Engine Warming Up**\n\n\
        The container is active but still preparing its inference server.\n\n\
        <options>\n\
        <option>Wait 20 seconds and retry automatically</option>\n\
        <option>Re-deploy with 16k context window (Recommended)</option>\n\
        <option>View recent error logs</option>\n\
        <option>Switch to Cloud AI</option>\n\
        </options>".to_string()
}
