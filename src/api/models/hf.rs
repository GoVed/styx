use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

#[derive(Debug, Deserialize)]
pub struct HfInspectQuery {
    pub repo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfQuantVariant {
    pub quant: String,
    pub filename: String,
    pub size_bytes: u64,
    pub size_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HfTreeItem {
    pub path: String,
    pub size: Option<u64>,
    #[serde(rename = "type")]
    pub item_type: String,
}

/// Normalizes raw input from users (including full HF URLs and tree links) into an `owner/repo` identifier.
pub fn sanitize_hf_repo_input(raw: &str) -> (String, Option<String>) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (String::new(), None);
    }

    let no_protocol = trimmed
        .trim_start_matches("https://")
        .trim_start_matches("http://");

    let no_domain = no_protocol
        .trim_start_matches("huggingface.co/")
        .trim_start_matches("hf.co/");

    // Check if quant was appended with a colon (e.g., repo:Q4_K_M or repo:model.gguf)
    let (path_only, selected_quant) = if let Some((r, q)) = no_domain.split_once(':') {
        (r.trim(), Some(q.trim().to_string()))
    } else {
        (no_domain, None)
    };

    let clean_path = path_only.split('?').next().unwrap_or(path_only);
    let parts: Vec<&str> = clean_path.split('/').filter(|s| !s.is_empty()).collect();

    let clean_repo = if parts.len() >= 2 {
        format!("{}/{}", parts[0], parts[1])
    } else {
        path_only.to_string()
    };

    (clean_repo, selected_quant)
}

/// Extracts a clean GGUF quantization identifier from a filename.
pub fn extract_gguf_quant(filename: &str) -> String {
    let stem = filename.strip_suffix(".gguf").unwrap_or(filename);
    let quant_patterns = [
        "BF16", "F16", "FP16", "Q8_0", "Q8_1", "Q6_K", "Q5_K_M", "Q5_K_S", "Q5_0", "Q5_1",
        "Q4_K_M", "Q4_K_S", "Q4_K_XL", "Q4_0", "Q4_1", "Q3_K_L", "Q3_K_M", "Q3_K_S",
        "Q2_K", "IQ4_NL", "IQ4_XS", "IQ3_XXS", "IQ3_M", "IQ2_XXS", "IQ2_M",
    ];

    for pat in quant_patterns {
        let upper_stem = stem.to_uppercase();
        if upper_stem.ends_with(&format!("-{}", pat))
            || upper_stem.ends_with(&format!("_{}", pat))
            || upper_stem.ends_with(&format!(".{}", pat))
            || upper_stem == pat
        {
            return pat.to_string();
        }
    }

    if let Some(pos) = stem.rfind(|c| c == '-' || c == '.' || c == '_') {
        let candidate = &stem[pos + 1..];
        if !candidate.is_empty() {
            return candidate.to_string();
        }
    }

    stem.to_string()
}

/// Handler for GET /api/models/hf-inspect?repo=...
/// Queries Hugging Face API to discover available GGUF quantizations and model metadata.
pub async fn inspect_hf_repo(
    Query(query): Query<HfInspectQuery>,
) -> impl IntoResponse {
    let (clean_repo, selected_quant) = sanitize_hf_repo_input(&query.repo);

    if clean_repo.is_empty() || clean_repo.starts_with('/') {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "success": false,
                "error": "A valid Hugging Face repository (e.g. owner/model-name) is required"
            })),
        );
    }

    let url = format!("https://huggingface.co/api/models/{clean_repo}/tree/main");
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    let res = match client
        .get(&url)
        .header("User-Agent", "Syndae-Agent-OS/1.0 (Local-Model-Inspector)")
        .send()
        .await
    {
        Ok(r) => r,
        Err(err) => {
            return (
                StatusCode::BAD_GATEWAY,
                Json(json!({
                    "success": false,
                    "error": format!("Failed to reach Hugging Face API: {err}")
                })),
            );
        }
    };

    if res.status() == StatusCode::NOT_FOUND {
        return (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "error": format!("Hugging Face repository '{clean_repo}' not found")
            })),
        );
    }

    if !res.status().is_success() {
        return (
            StatusCode::BAD_GATEWAY,
            Json(json!({
                "success": false,
                "error": format!("Hugging Face API returned error status {}", res.status())
            })),
        );
    }

    let items: Vec<HfTreeItem> = match res.json().await {
        Ok(data) => data,
        Err(err) => {
            return (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(json!({
                    "success": false,
                    "error": format!("Failed to parse repository files tree: {err}")
                })),
            );
        }
    };

    let mut gguf_quants: Vec<HfQuantVariant> = Vec::new();
    let mut is_awq = false;
    let mut is_gptq = false;
    let mut is_fp8 = false;

    for item in &items {
        let path_lower = item.path.to_lowercase();
        if path_lower.ends_with(".gguf") {
            let quant_tag = extract_gguf_quant(&item.path);
            let size_bytes = item.size.unwrap_or(0);
            let size_gb = ((size_bytes as f64) / (1024.0 * 1024.0 * 1024.0) * 100.0).round() / 100.0;
            gguf_quants.push(HfQuantVariant {
                quant: quant_tag,
                filename: item.path.clone(),
                size_bytes,
                size_gb,
            });
        } else if path_lower.contains("awq") {
            is_awq = true;
        } else if path_lower.contains("gptq") {
            is_gptq = true;
        } else if path_lower.contains("fp8") {
            is_fp8 = true;
        }
    }

    // Sort GGUF quants: popular sizes and bit depths first, or by size
    gguf_quants.sort_by(|a, b| {
        a.size_bytes.cmp(&b.size_bytes)
    });

    let is_gguf = !gguf_quants.is_empty();
    let recommended_engine = if is_gguf {
        "llama.cpp"
    } else {
        "vllm"
    };

    let recommended_quant = if is_gguf {
        selected_quant
            .or_else(|| {
                gguf_quants
                    .iter()
                    .find(|q| q.quant.eq_ignore_ascii_case("Q4_K_M") || q.quant.eq_ignore_ascii_case("Q4_0"))
                    .map(|q| q.quant.clone())
            })
            .or_else(|| gguf_quants.first().map(|q| q.quant.clone()))
            .unwrap_or_else(|| "Q4_K_M".to_string())
    } else if is_awq {
        "awq".to_string()
    } else if is_gptq {
        "gptq".to_string()
    } else if is_fp8 {
        "fp8".to_string()
    } else {
        "none".to_string()
    };

    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "repo": clean_repo,
            "is_gguf": is_gguf,
            "recommended_engine": recommended_engine,
            "recommended_quant": recommended_quant,
            "total_files": items.len(),
            "quants": gguf_quants,
        })),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_hf_repo_input() {
        assert_eq!(
            sanitize_hf_repo_input("https://huggingface.co/IFM/K2-Horizon-7B-GGUF/tree/main"),
            ("IFM/K2-Horizon-7B-GGUF".to_string(), None)
        );
        assert_eq!(
            sanitize_hf_repo_input("https://huggingface.co/IFM/K2-Horizon-7B-GGUF:Q4_K_M"),
            ("IFM/K2-Horizon-7B-GGUF".to_string(), Some("Q4_K_M".to_string()))
        );
        assert_eq!(
            sanitize_hf_repo_input("IFM/K2-Horizon-7B-GGUF"),
            ("IFM/K2-Horizon-7B-GGUF".to_string(), None)
        );
        assert_eq!(
            sanitize_hf_repo_input("  owner/model-7b  "),
            ("owner/model-7b".to_string(), None)
        );
    }

    #[test]
    fn test_extract_gguf_quant() {
        assert_eq!(extract_gguf_quant("K2-Horizon-7B-Q4_K_M.gguf"), "Q4_K_M");
        assert_eq!(extract_gguf_quant("K2-Horizon-7B-BF16.gguf"), "BF16");
        assert_eq!(extract_gguf_quant("model-q5_0.gguf"), "Q5_0");
        assert_eq!(extract_gguf_quant("Llama-3-8B.Q8_0.gguf"), "Q8_0");
    }
}
