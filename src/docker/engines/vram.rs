use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantizationRecommendation {
    pub target_gpu_vram_gb: usize,
    pub recommended_quantization: String,
    pub fits_comfortably: bool,
    pub estimated_weights_gb: f32,
    pub estimated_kv_cache_gb: f32,
    pub estimated_total_gb: f32,
    pub rationale: String,
}

pub fn recommend_quantization(
    model_size_b: f32,
    vram_gb: usize,
    context_tokens: usize,
) -> QuantizationRecommendation {
    // FP8 KV cache calculation
    let kv_cache_gb = if context_tokens >= 131072 {
        11.2
    } else if context_tokens >= 65536 {
        5.5
    } else if context_tokens >= 32768 {
        3.0
    } else {
        1.5
    };

    let (quant, weights_gb, rationale) = if vram_gb <= 12 {
        if model_size_b <= 9.0 {
            (
                "awq",
                model_size_b * 0.55,
                "AWQ 4-bit fits model weights (~5.0GB) + 64k FP8 KV cache (~5.5GB) within 12GB VRAM",
            )
        } else {
            (
                "k-quants",
                model_size_b * 0.45,
                "Requires GGUF Q4_K_M or partial offloading to run on 12GB VRAM",
            )
        }
    } else if vram_gb <= 16 {
        if model_size_b <= 14.0 {
            (
                "awq",
                model_size_b * 0.55,
                "AWQ 4-bit fits model weights (~7.7GB) + 64k FP8 KV cache (~5.5GB) within 16GB VRAM",
            )
        } else {
            (
                "awq",
                model_size_b * 0.55,
                "AWQ 4-bit recommended; tensor parallelism (TP=2) advised for 27B+ models on 16GB",
            )
        }
    } else if vram_gb <= 24 {
        if model_size_b <= 10.0 {
            (
                "fp8",
                model_size_b * 1.0,
                "FP8 8-bit fits easily in 24GB VRAM with full 64k context window",
            )
        } else if model_size_b <= 32.0 {
            (
                "awq",
                model_size_b * 0.55,
                "AWQ 4-bit fits model weights (~15-17GB) + 64k FP8 KV cache (~5.5GB) within 24GB VRAM",
            )
        } else {
            (
                "awq",
                model_size_b * 0.55,
                "AWQ 4-bit requires multi-GPU (TP=2) for models larger than 32B",
            )
        }
    } else {
        // 48GB, 80GB, or Multi-GPU
        if model_size_b <= 32.0 {
            (
                "fp8",
                model_size_b * 1.0,
                "FP8 8-bit fits natively with 64k context in 48GB+ VRAM",
            )
        } else {
            (
                "awq",
                model_size_b * 0.55,
                "AWQ 4-bit fits comfortably in 48GB VRAM with 64k context window",
            )
        }
    };

    let total_gb = weights_gb + kv_cache_gb;
    let fits = total_gb < (vram_gb as f32 * 0.95);

    QuantizationRecommendation {
        target_gpu_vram_gb: vram_gb,
        recommended_quantization: quant.to_string(),
        fits_comfortably: fits,
        estimated_weights_gb: (weights_gb * 10.0).round() / 10.0,
        estimated_kv_cache_gb: kv_cache_gb,
        estimated_total_gb: (total_gb * 10.0).round() / 10.0,
        rationale: rationale.to_string(),
    }
}
