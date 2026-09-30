use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::{doc, Index, IndexReader, IndexWriter, ReloadPolicy};
use tokio::sync::Mutex;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub path: String,
    pub category: String,
    pub title: String,
    pub snippet: String,
    pub score: f32,
}

#[derive(Clone)]
pub struct MemorySearchIndex {
    index: Index,
    reader: IndexReader,
    writer: Arc<Mutex<IndexWriter>>,
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

        Ok(Self {
            index,
            reader,
            writer: Arc::new(Mutex::new(writer)),
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
        let mut writer = self.writer.lock().await;

        // Delete any existing doc with this path
        let term = Term::from_field_text(self.f_path, rel_path);
        writer.delete_term(term);

        writer.add_document(doc!(
            self.f_path => rel_path,
            self.f_category => category,
            self.f_title => title,
            self.f_content => content,
        ))?;

        writer.commit()?;
        Ok(())
    }

    pub async fn remove_file(&self, rel_path: &str) -> Result<()> {
        let mut writer = self.writer.lock().await;
        let term = Term::from_field_text(self.f_path, rel_path);
        writer.delete_term(term);
        writer.commit()?;
        Ok(())
    }

    pub async fn sync_all_files(&self, memory_dir: &Path) -> Result<()> {
        if !memory_dir.exists() {
            return Ok(());
        }

        info!("Indexing all hierarchical memory files in {:?}", memory_dir);
        let mut files = Vec::new();
        walk_markdown_paths(memory_dir, memory_dir, &mut files);

        for (rel_path, category, title, content) in files {
            if let Err(e) = self.index_file(&rel_path, &category, &title, &content).await {
                error!("Failed to index file {}: {:?}", rel_path, e);
            }
        }

        Ok(())
    }

    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let searcher = self.reader.searcher();
        let query_parser =
            QueryParser::for_index(&self.index, vec![self.f_title, self.f_content]);

        // Fallback to fuzzy or wildcard if pure query fails parsing
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

            let path = retrieved_doc
                .get_first(self.f_path)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let category = retrieved_doc
                .get_first(self.f_category)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let title = retrieved_doc
                .get_first(self.f_title)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let content = retrieved_doc
                .get_first(self.f_content)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Generate clean contextual snippet
            let snippet = Self::extract_snippet(&content, query_str, 240);

            results.push(SearchResult {
                path,
                category,
                title,
                snippet,
                score,
            });
        }

        Ok(results)
    }

    fn extract_snippet(text: &str, query: &str, max_len: usize) -> String {
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

        let start = best_pos.saturating_sub(60);
        let end = (start + max_len).min(text.len());

        let prefix = if start > 0 { "..." } else { "" };
        let suffix = if end < text.len() { "..." } else { "" };

        let excerpt = &text[start..end];
        format!("{}{}{}", prefix, excerpt.replace('\n', " ").trim(), suffix)
    }
}

fn walk_markdown_paths(base: &Path, current: &Path, files: &mut Vec<(String, String, String, String)>) {
    if let Ok(entries) = std::fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str())
                && (name.starts_with('.') || name == "search_index") {
                    continue;
                }
            if path.is_dir() {
                walk_markdown_paths(base, &path, files);
            } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md")
                && let Ok(rel) = path.strip_prefix(base) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    let parts: Vec<&str> = rel_str.split('/').collect();
                    let category = if parts.len() > 1 { parts[0].to_string() } else { "uncategorized".to_string() };
                    let file_name = parts.last().unwrap_or(&"unknown.md").to_string();
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let title = content
                            .lines()
                            .find(|l| l.starts_with('#'))
                            .map(|l| l.trim_start_matches('#').trim().to_string())
                            .unwrap_or_else(|| file_name.clone());
                        files.push((rel_str, category, title, content));
                    }
                }
        }
    }
}
