use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::path::{Path, PathBuf};

pub const EMBEDDING_DIM: usize = 384;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorItem {
    pub path: String,
    pub category: String,
    pub title: String,
    pub snippet: String,
    pub media_type: String, // "text" or "image"
    pub embedding: Vec<f32>,
    pub content_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorSearchResult {
    pub path: String,
    pub category: String,
    pub title: String,
    pub snippet: String,
    pub media_type: String,
    pub score: f32, // [0.0, 1.0]
}

#[derive(Clone)]
pub struct FastVectorIndex {
    items: Vec<VectorItem>,
    storage_path: PathBuf,
}

impl FastVectorIndex {
    pub fn new(index_dir: &Path) -> Self {
        let storage_path = index_dir.join("memory_vectors.json");
        let mut items = Vec::new();
        if storage_path.exists()
            && let Ok(file) = File::open(&storage_path) {
                let reader = BufReader::new(file);
                if let Ok(loaded) = serde_json::from_reader(reader) {
                    items = loaded;
                }
            }

        Self {
            items,
            storage_path,
        }
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get_hash(&self, path: &str) -> Option<&str> {
        self.items
            .iter()
            .find(|item| item.path == path)
            .map(|item| item.content_hash.as_str())
    }

    pub fn upsert(&mut self, item: VectorItem) {
        if let Some(pos) = self.items.iter().position(|i| i.path == item.path) {
            self.items[pos] = item;
        } else {
            self.items.push(item);
        }
    }

    pub fn remove(&mut self, path: &str) {
        self.items.retain(|i| i.path != path);
    }

    pub fn search(&self, query_vec: &[f32], limit: usize, min_score: f32) -> Vec<VectorSearchResult> {
        if self.items.is_empty() || query_vec.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<VectorSearchResult> = self
            .items
            .iter()
            .map(|item| {
                let cos = cosine_similarity(query_vec, &item.embedding);
                let score = ((cos + 1.0) / 2.0).clamp(0.0, 1.0);
                VectorSearchResult {
                    path: item.path.clone(),
                    category: item.category.clone(),
                    title: item.title.clone(),
                    snippet: item.snippet.clone(),
                    media_type: item.media_type.clone(),
                    score,
                }
            })
            .filter(|r| r.score >= min_score)
            .collect();

        scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        if scored.len() > limit {
            scored.truncate(limit);
        }
        scored
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.storage_path.parent() {
            std::fs::create_dir_all(parent)
                .context("Failed to create vector storage directory")?;
        }
        let file = File::create(&self.storage_path)
            .context("Failed to open vector storage file for writing")?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &self.items)
            .context("Failed to serialize vector index")?;
        Ok(())
    }
}

#[inline]
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for i in 0..a.len() {
        dot += a[i] * b[i];
        norm_a += a[i] * a[i];
        norm_b += b[i] * b[i];
    }

    let denom = (norm_a.sqrt() * norm_b.sqrt()).max(1e-9);
    (dot / denom).clamp(-1.0, 1.0)
}
