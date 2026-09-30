use super::{detect_host_gpu_vendor, DeployModelRequest, EngineKind, GpuVendor};

impl DeployModelRequest {
    pub fn default_port(&self) -> u16 {
        self.port.unwrap_or(match self.engine {
            EngineKind::Vllm => 8000,
            EngineKind::LlamaCpp => 8080,
            EngineKind::Ollama => 11434,
        })
    }

    pub fn resolved_gpu_vendor(&self) -> GpuVendor {
        match self.gpu_vendor.unwrap_or(GpuVendor::Auto) {
            GpuVendor::Auto => detect_host_gpu_vendor(),
            other => other,
        }
    }

    pub fn image_name(&self) -> String {
        if let Some(custom) = &self.custom_image {
            let trimmed = custom.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }

        let vendor = self.resolved_gpu_vendor();
        match self.engine {
            EngineKind::Vllm => match vendor {
                GpuVendor::Amd => "vllm/vllm-openai-rocm:latest".to_string(),
                _ => "vllm/vllm-openai:latest".to_string(),
            },
            EngineKind::LlamaCpp => match vendor {
                GpuVendor::Amd => "ghcr.io/ggml-org/llama.cpp:server-rocm".to_string(),
                _ => "ghcr.io/ggerganov/llama.cpp:server".to_string(),
            },
            EngineKind::Ollama => match vendor {
                GpuVendor::Amd => "ollama/ollama:rocm".to_string(),
                _ => "ollama/ollama:latest".to_string(),
            },
        }
    }

    pub fn build_cmd_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        match self.engine {
            EngineKind::Vllm => {
                // In `vllm serve`, model is passed as the first positional argument
                args.push(self.hf_repo.clone());

                args.push("--port".to_string());
                args.push("8000".to_string());

                args.push("--host".to_string());
                args.push("0.0.0.0".to_string());

                let seqs = self.max_num_seqs.unwrap_or(2);
                args.push("--max-num-seqs".to_string());
                args.push(seqs.to_string());

                // Enable native tool calling parser for OpenAI function calls
                args.push("--enable-auto-tool-choice".to_string());
                args.push("--tool-call-parser".to_string());
                args.push("hermes".to_string());

                if let Some(ctx) = self.context_window {
                    args.push("--max-model-len".to_string());
                    args.push(ctx.to_string());
                }

                if let Some(kv) = &self.kv_cache_dtype {
                    if !kv.is_empty() && kv != "none" {
                        let vllm_type = match kv.as_str() {
                            "q4" | "q4_0" | "int4" | "int4_per_token_head" => "int4_per_token_head",
                            "q3" | "q3_k" | "q3_k_m" | "int3" | "turboquant_3bit_nc" => "turboquant_3bit_nc",
                            "q2" | "q2_k" | "int2" => "turboquant_3bit_nc",
                            "q8" | "q8_0" => "fp8",
                            "auto" | "fp16" | "f16" => "auto",
                            valid => valid,
                        };
                        args.push("--kv-cache-dtype".to_string());
                        args.push(vllm_type.to_string());
                    }
                } else if let Some(ctx) = self.context_window {
                    // If context length is 16k/32k/64k or larger, default to FP8 KV cache to prevent VRAM overflow
                    if ctx >= 16384 {
                        args.push("--kv-cache-dtype".to_string());
                        args.push("fp8".to_string());
                    }
                }

                if let Some(tp) = self.tensor_parallel_size
                    && tp > 1 {
                        args.push("--tensor-parallel-size".to_string());
                        args.push(tp.to_string());
                    }

                if let Some(util) = self.gpu_memory_utilization {
                    args.push("--gpu-memory-utilization".to_string());
                    args.push(format!("{:.2}", util));
                } else {
                    args.push("--gpu-memory-utilization".to_string());
                    args.push("0.95".to_string());
                }

                if let Some(quant) = &self.quantization
                    && !quant.is_empty() && quant != "none" {
                        args.push("--quantization".to_string());
                        args.push(quant.clone());
                    }

                // Speculative Decoding / Multi-Token Prediction (MTP) flags (Default: ON in presets)
                let mtp_enabled = self.enable_mtp.unwrap_or(true);
                if mtp_enabled {
                    let spec_model = self.speculative_model.as_deref().unwrap_or("[mtp]");
                    let num_tokens = self.num_speculative_tokens.unwrap_or(3);
                    if !spec_model.is_empty() {
                        args.push("--speculative-config".to_string());
                        if spec_model == "[mtp]" || spec_model == "mtp" {
                            args.push(format!("{{\"method\": \"mtp\", \"num_speculative_tokens\": {}}}", num_tokens));
                        } else {
                            args.push(format!("{{\"model\": \"{}\", \"num_speculative_tokens\": {}}}", spec_model, num_tokens));
                        }
                    }
                }

                // Multimodal / Image Input flags (Default: ON in presets)
                let vision_enabled = self.enable_vision.unwrap_or(true);
                if vision_enabled {
                    args.push("--limit-mm-per-prompt".to_string());
                    args.push("{\"image\": 2}".to_string());
                    args.push("--trust-remote-code".to_string());
                }

                if let Some(extras) = &self.extra_args {
                    for extra in extras {
                        args.push(extra.clone());
                    }
                }

                let vendor = self.resolved_gpu_vendor();
                if vendor == GpuVendor::Amd && !args.iter().any(|a| a == "--enforce-eager") {
                    args.push("--enforce-eager".to_string());
                }
            }
            EngineKind::LlamaCpp => {
                if self.hf_repo.starts_with('/') || self.hf_repo.ends_with(".gguf") {
                    let model_path = if !self.hf_repo.starts_with('/') {
                        format!("/models/{}", self.hf_repo)
                    } else {
                        self.hf_repo.clone()
                    };
                    args.push("-m".to_string());
                    args.push(model_path);
                } else {
                    args.push("-hf".to_string());
                    args.push(self.hf_repo.clone());
                }

                args.push("--host".to_string());
                args.push("0.0.0.0".to_string());

                args.push("--port".to_string());
                let p = self.port.unwrap_or(8080);
                args.push(p.to_string());

                args.push("-ngl".to_string());
                args.push("99".to_string()); // offload all layers to GPU by default

                if let Some(ctx) = self.context_window {
                    args.push("-c".to_string());
                    args.push(ctx.to_string());
                }

                if let Some(kv) = &self.kv_cache_dtype
                    && !kv.is_empty() && kv != "none" {
                        let llama_type = match kv.as_str() {
                            "q2" | "q2_k" => "q2_k",
                            "q3" | "q3_k" | "q3_k_m" | "turboquant_3bit_nc" => "q3_k_m",
                            "q4" | "q4_0" | "fp4" | "int4" | "int4_per_token_head" => "q4_0",
                            "q8" | "q8_0" | "fp8" | "fp8_e4m3" => "q8_0",
                            "auto" | "fp16" | "f16" => "f16",
                            custom => custom,
                        };
                        args.push("--cache-type-k".to_string());
                        args.push(llama_type.to_string());
                        args.push("--cache-type-v".to_string());
                        args.push(llama_type.to_string());
                    }

                // Speculative Decoding / MTP for llama.cpp
                let mtp_enabled = self.enable_mtp.unwrap_or(false);
                if mtp_enabled {
                    let mut draft_path = None;
                    if let Some(spec) = &self.speculative_model
                        && !spec.is_empty() && spec != "[mtp]" {
                            draft_path = Some(spec.clone());
                        }
                    if draft_path.is_none() {
                        if self.hf_repo.contains("Qwen3.8-27B") || self.hf_repo.contains("qwen3.8-27b") {
                            if self.hf_repo.contains("Uncensored") {
                                draft_path = Some("/models/Qwen3.8-27B-Uncensored-draft-Q4_0.gguf".to_string());
                            } else {
                                draft_path = Some("/models/mtp-Qwen3.8-27B-Q4_0.gguf".to_string());
                            }
                        } else if self.hf_repo.contains("gemma-4-12B") || self.hf_repo.contains("gemma-4-12b") {
                            draft_path = Some("/models/mtp-gemma-4-12B-it.gguf".to_string());
                        }
                    }

                    if let Some(draft) = draft_path {
                        args.push("-md".to_string());
                        args.push(draft);
                        args.push("--spec-draft-n-max".to_string());
                        let n_tokens = self.num_speculative_tokens.unwrap_or(3);
                        args.push(n_tokens.to_string());
                    }
                }

                // Image / Multimodal projector for llama.cpp (Default: ON in presets)
                let vision_enabled = self.enable_vision.unwrap_or(true);
                if vision_enabled {
                    if self.hf_repo.contains("Qwen3.5-9B") || self.hf_repo.contains("qwen3.5-9b") {
                        args.push("--mmproj".to_string());
                        args.push("/models/qwen3.5-9b-mmproj-F16.gguf".to_string());
                    } else if self.hf_repo.contains("Qwen3.8-27B") || self.hf_repo.contains("qwen3.8-27b") {
                        args.push("--mmproj".to_string());
                        if self.hf_repo.contains("Uncensored") {
                            args.push("/models/mmproj-Qwen3.8-27B-Uncensored-F16.gguf".to_string());
                        } else {
                            args.push("/models/mmproj-Qwen3.8-27B-f16.gguf".to_string());
                        }
                    }
                }

            }
            EngineKind::Ollama => {}
        }

        if let Some(extras) = &self.extra_args {
            args.extend(extras.iter().cloned());
        }

        args
    }

    pub fn build_env(&self) -> Vec<String> {
        let mut env = Vec::new();
        if let Some(token) = &self.hf_token
            && !token.is_empty() {
                env.push(format!("HUGGING_FACE_HUB_TOKEN={}", token));
                env.push(format!("HF_TOKEN={}", token));
            }

        let vendor = self.resolved_gpu_vendor();
        if let Some(gpus) = &self.gpu_devices
            && gpus != "all" && !gpus.is_empty() {
                match vendor {
                    GpuVendor::Amd => {
                        env.push(format!("HIP_VISIBLE_DEVICES={}", gpus));
                        env.push(format!("ROCR_VISIBLE_DEVICES={}", gpus));
                    }
                    _ => {
                        env.push(format!("CUDA_VISIBLE_DEVICES={}", gpus));
                    }
                }
            }

        env
    }

    pub fn generate_docker_cli_preview(&self) -> String {
        let port = self.default_port();
        let target_port = match self.engine {
            EngineKind::Vllm => 8000,
            EngineKind::LlamaCpp => 8080,
            EngineKind::Ollama => 11434,
        };

        let vendor = self.resolved_gpu_vendor();
        let gpu_flag = match vendor {
            GpuVendor::Amd => {
                "--device=/dev/kfd --device=/dev/dri --group-add=video --security-opt seccomp=unconfined --ipc=host"
            }
            GpuVendor::Nvidia => {
                match self.gpu_devices.as_deref().unwrap_or("all") {
                    "all" => "--gpus all --ipc=host",
                    devs => &format!("--gpus '\"device={}\"' --ipc=host", devs),
                }
            }
            GpuVendor::None => "",
            GpuVendor::Auto => "--ipc=host",
        };

        let env_flags = self.build_env().into_iter().map(|e| {
            let key = e.split('=').next().unwrap_or("");
            if key.contains("TOKEN") { format!(" -e {}=\"***\"", key) } else { format!(" -e \"{}\"", e) }
        }).collect::<String>();

        let cmd_args_raw = self.build_cmd_args();
        let formatted_cmd_args: Vec<String> = cmd_args_raw
            .into_iter()
            .map(|a| {
                if a.contains('{') || a.contains('}') || a.contains(' ') || a.contains('"') {
                    format!("'{}'", a)
                } else {
                    a
                }
            })
            .collect();
        let cmd_args = formatted_cmd_args.join(" ");
        let cmd_str = if cmd_args.is_empty() {
            String::new()
        } else {
            format!(" \\\n  {}", cmd_args)
        };

        let image = self.image_name();

        let mut parts = vec![
            format!("docker run -d --name styx-{}", self.name),
        ];
        if !gpu_flag.is_empty() {
            parts.push(format!("  {}", gpu_flag));
        }
        parts.push(format!("  -p {}:{}", port, target_port));
        parts.push("  -v ~/.cache/huggingface:/root/.cache/huggingface".to_string());
        parts.push("  -v ~/localLLM/models:/models:ro".to_string());
        if !env_flags.is_empty() {
            parts.push(format!(" {}", env_flags.trim()));
        }
        parts.push(format!("  {}{}", image, cmd_str));

        parts.join(" \\\n")
    }
}
