#[cfg(test)]
mod tests {
    use super::super::{
        recommend_quantization, DeployModelRequest, EngineKind, GpuVendor,
    };

    #[test]
    fn test_vllm_qwen35_9b_q4_131072_cli_preview() {
        let req = DeployModelRequest {
            name: "test-qwen".to_string(),
            engine: EngineKind::Vllm,
            hf_repo: "QuantTrio/Qwen3.5-9B-AWQ".to_string(),
            hf_token: None,
            context_window: Some(131072),
            gpu_vendor: Some(GpuVendor::Amd),
            gpu_devices: Some("all".to_string()),
            tensor_parallel_size: Some(1),
            gpu_memory_utilization: Some(0.90),
            quantization: Some("awq".to_string()),
            enable_mtp: Some(true),
            speculative_model: None,
            num_speculative_tokens: Some(3),
            enable_vision: Some(true),
            port: Some(8000),
            kv_cache_dtype: None,
            max_num_seqs: None,
            extra_args: None,
            custom_image: None,
        };

        let args = req.build_cmd_args();
        assert_eq!(args[0], "QuantTrio/Qwen3.5-9B-AWQ");
        assert!(args.contains(&"--max-model-len".to_string()));
        assert!(args.contains(&"131072".to_string()));
        assert!(args.contains(&"--kv-cache-dtype".to_string()));
        assert!(args.contains(&"fp8".to_string()));
        assert!(args.contains(&"--quantization".to_string()));
        assert!(args.contains(&"awq".to_string()));
        assert!(args.contains(&"--speculative-config".to_string()));
        assert!(args.contains(&"{\"method\": \"mtp\", \"num_speculative_tokens\": 3}".to_string()));
        assert!(args.contains(&"--limit-mm-per-prompt".to_string()));
        assert!(args.contains(&"{\"image\": 2}".to_string()));
        assert!(args.contains(&"--trust-remote-code".to_string()));
        assert!(args.contains(&"--enforce-eager".to_string()));

        let preview = req.generate_docker_cli_preview();
        assert!(preview.contains("docker run -d --name styx-test-qwen"));
        assert!(preview.contains("--device=/dev/kfd --device=/dev/dri --group-add=video --security-opt seccomp=unconfined --ipc=host"));
        assert!(preview.contains("vllm/vllm-openai-rocm:latest"));
        assert!(preview.contains("QuantTrio/Qwen3.5-9B-AWQ"));
        assert!(preview.contains("--enforce-eager"));
        assert!(preview.contains("'{\"method\": \"mtp\", \"num_speculative_tokens\": 3}'"));
        assert!(preview.contains("'{\"image\": 2}'"));
    }

    #[test]
    fn test_vllm_nvidia_gpu_preview() {
        let req = DeployModelRequest {
            name: "nvidia-qwen".to_string(),
            engine: EngineKind::Vllm,
            hf_repo: "QuantTrio/Qwen3.5-9B-AWQ".to_string(),
            hf_token: None,
            context_window: Some(65536),
            gpu_vendor: Some(GpuVendor::Nvidia),
            gpu_devices: Some("all".to_string()),
            tensor_parallel_size: Some(1),
            gpu_memory_utilization: None,
            quantization: Some("awq".to_string()),
            enable_mtp: Some(true),
            speculative_model: None,
            num_speculative_tokens: Some(3),
            enable_vision: Some(true),
            port: Some(8000),
            kv_cache_dtype: None,
            max_num_seqs: None,
            extra_args: None,
            custom_image: None,
        };

        let preview = req.generate_docker_cli_preview();
        assert!(preview.contains("--gpus all --ipc=host"));
        assert!(preview.contains("vllm/vllm-openai:latest"));
    }

    #[test]
    fn test_vllm_draft_model_speculative_config() {
        let req = DeployModelRequest {
            name: "draft-test".to_string(),
            engine: EngineKind::Vllm,
            hf_repo: "Qwen/Qwen2.5-32B-Instruct".to_string(),
            hf_token: None,
            context_window: Some(32768),
            gpu_vendor: Some(GpuVendor::Nvidia),
            gpu_devices: None,
            tensor_parallel_size: Some(1),
            gpu_memory_utilization: None,
            quantization: None,
            enable_mtp: Some(true),
            speculative_model: Some("Qwen/Qwen2.5-0.5B-Instruct".to_string()),
            num_speculative_tokens: Some(5),
            enable_vision: Some(false),
            port: None,
            kv_cache_dtype: None,
            max_num_seqs: None,
            extra_args: None,
            custom_image: None,
        };

        let args = req.build_cmd_args();
        assert!(args.contains(&"--speculative-config".to_string()));
        assert!(args.contains(&"{\"model\": \"Qwen/Qwen2.5-0.5B-Instruct\", \"num_speculative_tokens\": 5}".to_string()));

        let preview = req.generate_docker_cli_preview();
        assert!(preview.contains("'{\"model\": \"Qwen/Qwen2.5-0.5B-Instruct\", \"num_speculative_tokens\": 5}'"));
    }

    #[test]
    fn test_quantization_recommendation_16gb_vram() {
        let rec = recommend_quantization(9.0, 16, 65536);
        assert_eq!(rec.recommended_quantization, "awq");
        assert!(rec.fits_comfortably);
    }

    #[test]
    fn test_vllm_kv_cache_argument_mapping() {
        let base_req = DeployModelRequest {
            name: "vllm-kv-test".to_string(),
            engine: EngineKind::Vllm,
            hf_repo: "Qwen/Qwen2.5-Coder-7B".to_string(),
            hf_token: None,
            context_window: Some(32768),
            gpu_vendor: Some(GpuVendor::Nvidia),
            gpu_devices: None,
            tensor_parallel_size: Some(1),
            gpu_memory_utilization: None,
            quantization: None,
            enable_mtp: None,
            speculative_model: None,
            num_speculative_tokens: None,
            enable_vision: None,
            port: None,
            kv_cache_dtype: Some("int4_per_token_head".to_string()),
            max_num_seqs: None,
            extra_args: None,
            custom_image: None,
        };

        let args = base_req.build_cmd_args();
        let kv_idx = args.iter().position(|a| a == "--kv-cache-dtype").unwrap();
        assert_eq!(args[kv_idx + 1], "int4_per_token_head");

        let mut req2 = base_req.clone();
        req2.kv_cache_dtype = Some("q4".to_string());
        let args2 = req2.build_cmd_args();
        let kv_idx2 = args2.iter().position(|a| a == "--kv-cache-dtype").unwrap();
        assert_eq!(args2[kv_idx2 + 1], "int4_per_token_head");

        let mut req3 = base_req.clone();
        req3.kv_cache_dtype = Some("turboquant_3bit_nc".to_string());
        let args3 = req3.build_cmd_args();
        let kv_idx3 = args3.iter().position(|a| a == "--kv-cache-dtype").unwrap();
        assert_eq!(args3[kv_idx3 + 1], "turboquant_3bit_nc");
    }

    #[test]
    fn test_llamacpp_kv_cache_argument_mapping() {
        let req = DeployModelRequest {
            name: "llama-kv-test".to_string(),
            engine: EngineKind::LlamaCpp,
            hf_repo: "Qwen/Qwen2.5-Coder-7B-GGUF".to_string(),
            hf_token: None,
            context_window: Some(32768),
            gpu_vendor: Some(GpuVendor::Nvidia),
            gpu_devices: None,
            tensor_parallel_size: None,
            gpu_memory_utilization: None,
            quantization: None,
            enable_mtp: None,
            speculative_model: None,
            num_speculative_tokens: None,
            enable_vision: None,
            port: None,
            kv_cache_dtype: Some("q4_0".to_string()),
            max_num_seqs: None,
            extra_args: None,
            custom_image: None,
        };

        let args = req.build_cmd_args();
        let k_idx = args.iter().position(|a| a == "--cache-type-k").unwrap();
        let v_idx = args.iter().position(|a| a == "--cache-type-v").unwrap();
        assert_eq!(args[k_idx + 1], "q4_0");
        assert_eq!(args[v_idx + 1], "q4_0");
    }

    #[test]
    fn test_llamacpp_local_gguf_args() {
        let req = DeployModelRequest {
            name: "qwen9b-local".to_string(),
            engine: EngineKind::LlamaCpp,
            hf_repo: "Qwen3.5-9B-UD-Q4_K_XL.gguf".to_string(),
            hf_token: None,
            context_window: Some(131072),
            gpu_vendor: Some(GpuVendor::Amd),
            gpu_devices: None,
            tensor_parallel_size: None,
            gpu_memory_utilization: None,
            quantization: None,
            enable_mtp: Some(false),
            speculative_model: None,
            num_speculative_tokens: None,
            enable_vision: Some(true),
            port: Some(8080),
            kv_cache_dtype: Some("q8_0".to_string()),
            max_num_seqs: None,
            extra_args: None,
            custom_image: None,
        };

        let args = req.build_cmd_args();
        assert!(args.contains(&"-m".to_string()));
        let m_idx = args.iter().position(|a| a == "-m").unwrap();
        assert_eq!(args[m_idx + 1], "/models/Qwen3.5-9B-UD-Q4_K_XL.gguf");

        assert!(args.contains(&"--mmproj".to_string()));
        let proj_idx = args.iter().position(|a| a == "--mmproj").unwrap();
        assert_eq!(args[proj_idx + 1], "/models/qwen3.5-9b-mmproj-F16.gguf");

        assert!(args.contains(&"-c".to_string()));
        let c_idx = args.iter().position(|a| a == "-c").unwrap();
        assert_eq!(args[c_idx + 1], "131072");

        // Verify obsolete or absent draft flags are never passed
        assert!(!args.contains(&"--draft-max".to_string()));
        assert!(!args.contains(&"-md".to_string()));
        assert!(!args.contains(&"--spec-draft-n-max".to_string()));
    }

    #[test]
    fn test_engine_kind_serde_aliases() {
        let de_vllm: EngineKind = serde_json::from_str("\"vllm\"").unwrap();
        assert_eq!(de_vllm, EngineKind::Vllm);
        let de_vllm_cap: EngineKind = serde_json::from_str("\"vLLM\"").unwrap();
        assert_eq!(de_vllm_cap, EngineKind::Vllm);

        let de_llamacpp: EngineKind = serde_json::from_str("\"llamacpp\"").unwrap();
        assert_eq!(de_llamacpp, EngineKind::LlamaCpp);
        let de_llama_cpp: EngineKind = serde_json::from_str("\"llama_cpp\"").unwrap();
        assert_eq!(de_llama_cpp, EngineKind::LlamaCpp);
        let de_llama_dot: EngineKind = serde_json::from_str("\"llama.cpp\"").unwrap();
        assert_eq!(de_llama_dot, EngineKind::LlamaCpp);

        let de_ollama: EngineKind = serde_json::from_str("\"ollama\"").unwrap();
        assert_eq!(de_ollama, EngineKind::Ollama);
    }

    #[test]
    fn test_gpu_vendor_serde_aliases() {
        let de_amd: GpuVendor = serde_json::from_str("\"amd\"").unwrap();
        assert_eq!(de_amd, GpuVendor::Amd);
        let de_rocm: GpuVendor = serde_json::from_str("\"rocm\"").unwrap();
        assert_eq!(de_rocm, GpuVendor::Amd);
        let de_cuda: GpuVendor = serde_json::from_str("\"cuda\"").unwrap();
        assert_eq!(de_cuda, GpuVendor::Nvidia);
        let de_cpu: GpuVendor = serde_json::from_str("\"cpu\"").unwrap();
        assert_eq!(de_cpu, GpuVendor::None);
    }
}
