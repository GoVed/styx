use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy};
use tokio::sync::Mutex;
use tracing::{error, info};

use super::embeddings::SemanticEmbedder;
use super::images::{is_image_extension, scan_image_files, walk_markdown_paths};
use super::vector::{FastVectorIndex, VectorItem};

fn default_media_type() -> String {
    "text".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub category: String,
    pub title: String,
    pub snippet: String,
    #[serde(default = "default_media_type")]
    pub media_type: String,
    pub score: f32,
}

#[derive(Clone)]
pub struct MemorySearchIndex {
    index: Index,
    reader: IndexReader,
    writer: Arc<Mutex<IndexWriter>>,
    vector_index: Arc<Mutex<FastVectorIndex>>,
    embedder: Arc<SemanticEmbedder>,
    f_path: Field,
    f_category: Field,
    f_title: Field,
    f_content: Field,
}

impl MemorySearchIndex {
    pub fn new(index_path: &Path) -> Result<Self> {
        let mut schema_builder = Schema::builder();
        let f_path = schema_builder.add_text_field("path", STRING | STORED);
        let f_category = schema_builder.add_text_field("category", STRING | STORED);
        let f_title = schema_builder.add_text_field("title", TEXT | STORED);
        let f_content = schema_builder.add_text_field("content", TEXT | STORED);
        let schema = schema_builder.build();

        let index = if index_path.exists() && index_path.join("meta.json").exists() {
            Index::open_in_dir(index_path).context("Failed to open existing tantivy index")?
        } else {
            std::fs::create_dir_all(index_path).context("Failed to create tantivy dir")?;
            Index::create_in_dir(index_path, schema.clone())
                .context("Failed to create new tantivy index")?
        };

        let writer = index
            .writer(50_000_000)
            .context("Failed to create tantivy index writer")?;
        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::OnCommitWithDelay)
            .try_into()
            .context("Failed to create tantivy reader")?;

        let vector_index = Arc::new(Mutex::new(FastVectorIndex::new(index_path)));
        let embedder = Arc::new(SemanticEmbedder::new());

        Ok(Self {
            index,
            reader,
            writer: Arc::new(Mutex::new(writer)),
            vector_index,
            embedder,
            f_path,
            f_category,
            f_title,
            f_content,
        })
    }

    pub async fn index_file(
        &self,
        rel_path: &str,
        category: &str,
        title: &str,
        content: &str,
    ) -> Result<()> {
        let media_type = if is_image_path(rel_path) { "image" } else { "text" };
        let content_hash = SemanticEmbedder::compute_hash(content);

        // 1. Tantivy BM25 indexing
        {
            let mut writer = self.writer.lock().await;
            let term = Term::from_field_text(self.f_path, rel_path);
            writer.delete_term(term);
            writer.add_document(doc!(
                self.f_path => rel_path,
                self.f_category => category,
                self.f_title => title,
                self.f_content => content,
            ))?;
            writer.commit()?;
        }

        // 2. Semantic vector indexing with hash-caching
        let mut v_idx = self.vector_index.lock().await;
        if v_idx.get_hash(rel_path) != Some(&content_hash) {
            let embedding = self.embedder.embed(content).await;
            let snippet = Self::extract_snippet(content, title, 240);
            v_idx.upsert(VectorItem {
                path: rel_path.to_string(),
                category: category.to_string(),
                title: title.to_string(),
                snippet,
                media_type: media_type.to_string(),
                embedding,
                content_hash,
            });
            let _ = v_idx.save();
        }

        Ok(())
    }

    pub async fn remove_file(&self, rel_path: &str) -> Result<()> {
        {
            let mut writer = self.writer.lock().await;
            let term = Term::from_field_text(self.f_path, rel_path);
            writer.delete_term(term);
            writer.commit()?;
        }
        {
            let mut v_idx = self.vector_index.lock().await;
            v_idx.remove(rel_path);
            let _ = v_idx.save();
        }
        Ok(())
    }

    pub async fn sync_all_files(&self, memory_dir: &Path) -> Result<()> {
        if !memory_dir.exists() {
            return Ok(());
        }

        info!("Indexing all hierarchical memory and media files in {:?}", memory_dir);
        let mut files = Vec::new();
        walk_markdown_paths(memory_dir, memory_dir, &mut files);

        for (rel_path, category, title, content) in files {
            if let Err(e) = self.index_file(&rel_path, &category, &title, &content).await {
                error!("Failed to index file {}: {:?}", rel_path, e);
            }
        }

        let mut image_items = Vec::new();
        scan_image_files(memory_dir, memory_dir, &mut image_items);
        for img in image_items {
            if let Err(e) = self
                .index_file(&img.rel_path, &img.category, &img.title, &img.description)
                .await
            {
                error!("Failed to index image {}: {:?}", img.rel_path, e);
            }
        }

        Ok(())
    }

    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let bm25_results = self.search_tantivy(query_str, limit * 2).unwrap_or_default();
        let query_vec = SemanticEmbedder::fast_semantic_encode(query_str);

        let vector_results = if let Ok(v_idx) = self.vector_index.try_lock() {
            v_idx.search(&query_vec, limit * 2, 0.05)
        } else {
            Vec::new()
        };

        // Reciprocal rank fusion and hybrid scoring
        let mut merged: HashMap<String, SearchResult> = HashMap::new();
        let query_lower = query_str.to_lowercase();
        let query_tokens: Vec<&str> = query_lower.split_whitespace().collect();

        let max_bm25 = bm25_results
            .iter()
            .map(|r| r.score)
            .fold(0.0f32, f32::max)
            .max(1.0);

        for bm in bm25_results {
            let norm_bm25 = (bm.score / max_bm25).clamp(0.0, 1.0);
            let entity_boost = calculate_entity_boost(&bm.path, &bm.title, &query_tokens);
            let combined = (norm_bm25 * 0.45) + entity_boost;
            let media_type = if is_image_path(&bm.path) { "image".to_string() } else { "text".to_string() };
            merged.insert(
                bm.path.clone(),
                SearchResult {
                    path: bm.path,
                    category: bm.category,
                    title: bm.title,
                    snippet: bm.snippet,
                    media_type,
                    score: combined,
                },
            );
        }

        for vr in vector_results {
            let entity_boost = calculate_entity_boost(&vr.path, &vr.title, &query_tokens);
            let sem_weighted = (vr.score * 0.55) + entity_boost;

            if let Some(existing) = merged.get_mut(&vr.path) {
                existing.score += vr.score * 0.55;
                if existing.snippet.len() < vr.snippet.len() {
                    existing.snippet = vr.snippet;
                }
            } else {
                merged.insert(
                    vr.path.clone(),
                    SearchResult {
                        path: vr.path,
                        category: vr.category,
                        title: vr.title,
                        snippet: vr.snippet,
                        media_type: vr.media_type,
                        score: sem_weighted,
                    },
                );
            }
        }

        let mut results: Vec<SearchResult> = merged.into_values().collect();
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        if results.len() > limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    fn search_tantivy(&self, query_str: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let searcher = self.reader.searcher();
        let query_parser = QueryParser::for_index(&self.index, vec![self.f_title, self.f_content]);

        let query = match query_parser.parse_query(query_str) {
            Ok(q) => q,
            Err(_) => {
                let sanitized: String = query_str
                    .chars()
                    .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                    .collect();
                if sanitized.trim().is_empty() {
                    return Ok(Vec::new());
                }
                query_parser
                    .parse_query(&sanitized)
                    .unwrap_or_else(|_| Box::new(tantivy::query::AllQuery))
            }
        };

        let top_docs = searcher.search(&query, &TopDocs::with_limit(limit))?;
        let mut results = Vec::new();

        for (score, doc_address) in top_docs {
            let retrieved_doc: TantivyDocument = searcher.doc(doc_address)?;
            let path = retrieved_doc.get_first(self.f_path).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let category = retrieved_doc.get_first(self.f_category).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let title = retrieved_doc.get_first(self.f_title).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let content = retrieved_doc.get_first(self.f_content).and_then(|v| v.as_str()).unwrap_or("").to_string();
            let snippet = Self::extract_snippet(&content, query_str, 240);
            let media_type = if is_image_path(&path) { "image".to_string() } else { "text".to_string() };

            results.push(SearchResult { path, category, title, snippet, media_type, score });
        }

        Ok(results)
    }

    pub fn extract_snippet(text: &str, query: &str, max_len: usize) -> String {
        let lower_text = text.to_lowercase();
        let query_terms: Vec<String> = query
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .filter(|s| s.len() > 2)
            .collect();

        let mut best_pos = 0;
        for term in &query_terms {
            if let Some(pos) = lower_text.find(term) {
                best_pos = pos;
                break;
            }
        }

        let mut start = best_pos.saturating_sub(60);
        while start > 0 && !text.is_char_boundary(start) {
            start = start.saturating_sub(1);
        }
        let mut end = (start + max_len).min(text.len());
        while end < text.len() && !text.is_char_boundary(end) {
            end = end.saturating_sub(1);
        }
        let prefix = if start > 0 { "..." } else { "" };
        let suffix = if end < text.len() { "..." } else { "" };
        let excerpt = &text[start..end];
        format!("{}{}{}", prefix, excerpt.replace('\n', " ").trim(), suffix)
    }
}

fn calculate_entity_boost(path: &str, title: &str, query_tokens: &[&str]) -> f32 {
    let path_lower = path.to_lowercase();
    let title_lower = title.to_lowercase();
    let mut boost = 0.0f32;

    for &tok in query_tokens {
        if tok.len() < 3 {
            continue;
        }
        if path_lower.contains(tok) {
            boost += 0.15;
        }
        if title_lower.contains(tok) {
            boost += 0.20;
        }
    }

    boost.min(0.50)
}

fn is_image_path(path: &str) -> bool {
    let ext = path.rsplit('.').next().unwrap_or("");
    is_image_extension(ext)
}
