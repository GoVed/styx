pub mod args;
pub mod presets;
pub mod vram;

#[cfg(test)]
mod tests;

pub use presets::get_preset_templates;
pub use vram::recommend_quantization;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineKind {
    #[serde(rename = "vllm", alias = "vLLM", alias = "VLLM")]
    Vllm,
    #[serde(rename = "llama_cpp", alias = "llamacpp", alias = "llama.cpp", alias = "llama")]
    LlamaCpp,
    #[serde(rename = "ollama", alias = "Ollama")]
    Ollama,
}

impl std::fmt::Display for EngineKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineKind::Vllm => write!(f, "vLLM"),
            EngineKind::LlamaCpp => write!(f, "llama.cpp"),
            EngineKind::Ollama => write!(f, "Ollama"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GpuVendor {
    #[default]
    #[serde(rename = "auto", alias = "Auto")]
    Auto,
    #[serde(rename = "nvidia", alias = "Nvidia", alias = "cuda", alias = "CUDA")]
    Nvidia,
    #[serde(rename = "amd", alias = "Amd", alias = "AMD", alias = "rocm", alias = "ROCm")]
    Amd,
    #[serde(rename = "none", alias = "None", alias = "cpu", alias = "CPU")]
    None,
}

impl std::fmt::Display for GpuVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GpuVendor::Auto => write!(f, "auto"),
            GpuVendor::Nvidia => write!(f, "nvidia"),
            GpuVendor::Amd => write!(f, "amd"),
            GpuVendor::None => write!(f, "none"),
        }
    }
}

pub fn detect_host_gpu_vendor() -> GpuVendor {
    // 1. Check sysfs DRM vendor IDs (accessible inside and outside containers)
    if let Ok(entries) = std::fs::read_dir("/sys/class/drm") {
        for entry in entries.flatten() {
            let vendor_path = entry.path().join("device/vendor");
            if let Ok(vendor_str) = std::fs::read_to_string(&vendor_path) {
                let trimmed = vendor_str.trim().to_lowercase();
                if trimmed == "0x1002" {
                    return GpuVendor::Amd;
                } else if trimmed == "0x10de" {
                    return GpuVendor::Nvidia;
                }
            }
        }
    }

    // 2. Check device nodes
    if std::path::Path::new("/dev/kfd").exists() || std::path::Path::new("/dev/dri").exists() {
        return GpuVendor::Amd;
    }
    if std::path::Path::new("/dev/nvidia0").exists() {
        return GpuVendor::Nvidia;
    }

    // 3. Check CLI tools
    if let Ok(output) = std::process::Command::new("which").arg("nvidia-smi").output()
        && output.status.success() {
            return GpuVendor::Nvidia;
        }
    if let Ok(output) = std::process::Command::new("which").arg("rocm-smi").output()
        && output.status.success() {
            return GpuVendor::Amd;
        }

    GpuVendor::None
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployModelRequest {
    pub name: String,
    pub engine: EngineKind,
    pub hf_repo: String,
    pub hf_token: Option<String>,
    pub context_window: Option<usize>,
    pub speculative_model: Option<String>,
    pub num_speculative_tokens: Option<usize>,
    pub gpu_devices: Option<String>, // "all", "0", "0,1"
    #[serde(default)]
    pub gpu_vendor: Option<GpuVendor>, // Auto (default), Nvidia, Amd, None
    #[serde(default)]
    pub custom_image: Option<String>,
    pub tensor_parallel_size: Option<usize>,
    pub gpu_memory_utilization: Option<f32>,
    pub quantization: Option<String>, // "awq", "gptq", "fp8", "bitsandbytes"
    pub port: Option<u16>,
    #[serde(default)]
    pub enable_mtp: Option<bool>,
    #[serde(default)]
    pub enable_vision: Option<bool>,
    #[serde(default)]
    pub kv_cache_dtype: Option<String>,
    #[serde(default)]
    pub max_num_seqs: Option<usize>,
    pub extra_args: Option<Vec<String>>,
}
