use reqwest::Client;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::time::Duration;
use tracing::warn;

use super::vector::EMBEDDING_DIM;

#[derive(Clone, Default)]
pub struct SemanticEmbedder {
    client: Client,
    remote_url: Option<String>,
}

impl SemanticEmbedder {
    pub fn new() -> Self {
        let remote_url = std::env::var("STYX_EMBEDDING_URL")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let client = Client::builder()
            .timeout(Duration::from_secs(4))
            .build()
            .unwrap_or_default();

        Self { client, remote_url }
    }

    pub fn compute_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hex::encode(hasher.finalize())
    }

    pub async fn embed(&self, text: &str) -> Vec<f32> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return vec![0.0; EMBEDDING_DIM];
        }

        if let Some(ref url) = self.remote_url
            && let Ok(vec) = self.fetch_remote_embedding(url, trimmed).await {
                return vec;
            }

        Self::fast_semantic_encode(trimmed)
    }

    async fn fetch_remote_embedding(&self, url: &str, text: &str) -> Result<Vec<f32>, ()> {
        let payload = json!({
            "input": text,
            "model": "text-embedding-3-small"
        });

        let resp = self.client
            .post(url)
            .json(&payload)
            .send()
            .await
            .map_err(|e| {
                warn!("Remote embedding request failed: {}", e);
            })?;

        if !resp.status().is_success() {
            return Err(());
        }

        let body: Value = resp.json().await.map_err(|_| ())?;
        let vec_opt = body.get("data")
            .and_then(|d| d.get(0))
            .and_then(|item| item.get("embedding"))
            .and_then(|e| e.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_f64().map(|f| f as f32))
                    .collect::<Vec<f32>>()
            });

        if let Some(vec) = vec_opt
            && !vec.is_empty() {
                return Ok(normalize_vector(&vec));
            }

        Err(())
    }

    pub fn fast_semantic_encode(text: &str) -> Vec<f32> {
        let mut vec = vec![0.0f32; EMBEDDING_DIM];
        let lines: Vec<&str> = text.lines().collect();

        for line in lines {
            let is_header = line.trim_start().starts_with('#');
            let base_multiplier = if is_header { 2.5 } else { 1.0 };

            for raw_word in line.split_whitespace() {
                let clean: String = raw_word
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-')
                    .collect();

                if clean.is_empty() {
                    continue;
                }

                let lower = clean.to_lowercase();
                let is_stopword = matches!(
                    lower.as_str(),
                    "the" | "is" | "a" | "an" | "and" | "or" | "in" | "on" | "of" | "to" | "for" | "with" | "at" | "by"
                );

                let is_capitalized = clean.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                let weight = if is_stopword {
                    0.15 * base_multiplier
                } else if is_capitalized {
                    2.0 * base_multiplier
                } else {
                    1.0 * base_multiplier
                };

                accumulate_ngram_hashes(&lower, weight, &mut vec);
            }
        }

        normalize_vector(&vec)
    }
}

fn accumulate_ngram_hashes(word: &str, weight: f32, vec: &mut [f32]) {
    accumulate_token_hash(word, weight * 1.5, vec);

    let chars: Vec<char> = word.chars().collect();
    let len = chars.len();
    for n in 3..=5 {
        if len >= n {
            for start in 0..=(len - n) {
                let ngram: String = chars[start..start + n].iter().collect();
                accumulate_token_hash(&ngram, weight * 0.7, vec);
            }
        }
    }
}

fn accumulate_token_hash(token: &str, weight: f32, vec: &mut [f32]) {
    let mut h1: u64 = 0xcbf29ce484222325;
    let mut h2: u64 = 0x84222325cbf29ce4;

    for byte in token.bytes() {
        h1 = (h1 ^ (byte as u64)).wrapping_mul(0x100000001b3);
        h2 = (h2 ^ (byte as u64)).wrapping_mul(0x100000001b3 ^ 0x5555555555555555);
    }

    let idx1 = (h1 as usize) % EMBEDDING_DIM;
    let sign1 = if (h2 & 1) == 0 { 1.0f32 } else { -1.0f32 };
    vec[idx1] += weight * sign1;

    let idx2 = ((h1 >> 16) as usize) % EMBEDDING_DIM;
    let sign2 = if (h2 & 2) == 0 { 1.0f32 } else { -1.0f32 };
    vec[idx2] += (weight * 0.5) * sign2;
}

fn normalize_vector(v: &[f32]) -> Vec<f32> {
    let mut norm_sq = 0.0f32;
    for x in v {
        norm_sq += x * x;
    }
    let norm = norm_sq.sqrt();
    if norm < 1e-9 {
        return vec![0.0; v.len()];
    }
    v.iter().map(|x| x / norm).collect()
}
