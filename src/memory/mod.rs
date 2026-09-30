pub mod merge;
pub mod search;

use anyhow::{bail, Context, Result};
use search::{MemorySearchIndex, SearchResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFileNode {
    pub path: String,       // relative path e.g. "skills/custom_task.md"
    pub category: String,   // "core", "skills", "scratchpad"
    pub filename: String,   // "custom_task.md"
    pub title: String,      // "# Skillset: Custom Task" or filename
    pub size_bytes: u64,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct MemoryManager {
    base_dir: PathBuf,
    search_index: Arc<MemorySearchIndex>,
}

impl MemoryManager {
    pub fn new(base_dir: PathBuf, search_index: Arc<MemorySearchIndex>) -> Self {
        Self {
            base_dir,
            search_index,
        }
    }

    #[allow(dead_code)]
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    #[allow(dead_code)]
    pub fn search_index(&self) -> Arc<MemorySearchIndex> {
        self.search_index.clone()
    }

    pub fn ensure_directories(&self) -> Result<()> {
        let defaults = ["core", "skills", "scratchpad", "dictionary", "people", "groups"];
        for d in defaults {
            std::fs::create_dir_all(self.base_dir.join(d))?;
        }
        Ok(())
    }

    pub fn list_tree(&self) -> Result<Vec<MemoryFileNode>> {
        let mut list = Vec::new();
        scan_dir_recursive(&self.base_dir, &self.base_dir, &mut list);
        list.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(list)
    }

    pub fn read_file(&self, rel_path: &str) -> Result<String> {
        let safe_rel = sanitize_rel_path(rel_path)?;
        let full_path = self.base_dir.join(&safe_rel);
        if !full_path.exists() {
            bail!("Memory file not found: {}", rel_path);
        }
        let content = std::fs::read_to_string(full_path)
            .with_context(|| format!("Failed to read memory file: {}", rel_path))?;
        Ok(content)
    }

    pub async fn write_file(
        &self,
        rel_path: &str,
        content: &str,
        section: Option<&str>,
    ) -> Result<()> {
        let safe_rel = sanitize_rel_path(rel_path)?;
        let full_path = self.base_dir.join(&safe_rel);

        let final_content = if full_path.exists() {
            let existing = std::fs::read_to_string(&full_path).unwrap_or_default();
            merge::merge_markdown_documents(&existing, content, section)
        } else if let Some(sec) = section {
            format!("## {}\n{}\n", sec.trim().trim_start_matches('#').trim(), content.trim())
        } else {
            content.to_string()
        };

        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&full_path, &final_content)
            .with_context(|| format!("Failed to write memory file: {}", rel_path))?;

        let parts: Vec<&str> = safe_rel.split('/').collect();
        let category = parts.first().copied().unwrap_or("skills");
        let filename = parts.last().copied().unwrap_or("note.md");
        let title = final_content
            .lines()
            .find(|l| l.starts_with('#'))
            .map(|l| l.trim_start_matches('#').trim().to_string())
            .unwrap_or_else(|| filename.to_string());

        // Re-index in Tantivy
        let _ = self
            .search_index
            .index_file(&safe_rel, category, &title, &final_content)
            .await;

        Ok(())
    }

    pub async fn create_file(&self, category: &str, filename: &str, content: &str) -> Result<String> {
        let clean_cat = category.trim().trim_matches('/');
        if clean_cat.is_empty()
            || clean_cat.starts_with('.')
            || !clean_cat.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            bail!("Invalid category '{}'. Must contain alphanumeric characters, underscores, or dashes", category);
        }

        let clean_filename = if filename.ends_with(".md") {
            filename.to_string()
        } else {
            format!("{}.md", filename)
        };

        let rel_path = format!("{}/{}", clean_cat, clean_filename);
        self.write_file(&rel_path, content, None).await?;
        Ok(rel_path)
    }

    pub async fn delete_file(&self, rel_path: &str) -> Result<()> {
        let safe_rel = sanitize_rel_path(rel_path)?;
        let full_path = self.base_dir.join(&safe_rel);
        if full_path.exists() {
            std::fs::remove_file(full_path)?;
            let _ = self.search_index.remove_file(&safe_rel).await;
        }
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        self.search_index.search(query, limit)
    }

    /// Load all core guidelines and active scratchpads as system context
    pub fn build_system_context(&self) -> String {
        let mut context = String::new();

        // 1. Read core files
        for core_file in ["core/system_instructions.md", "core/user_profile.md"] {
            if let Ok(content) = self.read_file(core_file) {
                context.push_str(&format!("\n\n=== MEMORY: {} ===\n{}\n", core_file, content.trim()));
            }
        }

        // 2. Read active scratchpads
        for scratch in ["scratchpad/daily_log.md", "scratchpad/active_projects.md"] {
            if let Ok(content) = self.read_file(scratch) {
                context.push_str(&format!("\n\n=== SCRATCHPAD: {} ===\n{}\n", scratch, content.trim()));
            }
        }

        // 3. Read active user dictionary files dynamically
        let dict_dir = self.base_dir.join("dictionary");
        if let Ok(entries) = std::fs::read_dir(&dict_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md")
                    && let Ok(content) = std::fs::read_to_string(&path) {
                        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("notes.md");
                        context.push_str(&format!("\n\n=== USER DICTIONARY: {} ===\n{}\n", name, content.trim()));
                    }
            }
        }

        context
    }

    /// Retrieve contextual memory for an entity (contact, group, or dictionary topic)
    #[allow(dead_code)]
    pub fn get_entity_context(&self, entity_name: &str) -> Option<String> {
        let clean = entity_name.trim().to_lowercase().replace([' ', '@', '.', ':', '+', '-'], "_");
        let raw_clean = entity_name.trim().to_lowercase();
        for cat in ["people", "groups", "dictionary"] {
            for candidate_name in [&clean, &raw_clean] {
                let candidate = format!("{}/{}.md", cat, candidate_name);
                if let Ok(content) = self.read_file(&candidate) {
                    return Some(format!("\n=== MEMORY CONTEXT: {} ===\n{}\n", candidate, content.trim()));
                }
            }
        }
        None
    }

    /// Check whether a communication channel or group (e.g. "15551234567-1600000000@g.us")
    /// is explicitly marked to be ignored or muted in user_profile.md or system_instructions.md
    pub fn is_channel_ignored(&self, channel_id: &str) -> bool {
        let clean_cid = channel_id.trim().to_lowercase();
        if clean_cid.is_empty() {
            return false;
        }

        for file_path in ["core/user_profile.md", "core/system_instructions.md"] {
            if let Ok(content) = self.read_file(file_path) {
                for line in content.lines() {
                    let l = line.to_lowercase();
                    if (l.contains("ignore") || l.contains("mute") || l.contains("block") || l.contains("suppress"))
                        && l.contains(&clean_cid)
                    {
                        return true;
                    }
                }
            }
        }
        false
    }
}

fn sanitize_rel_path(path: &str) -> Result<String> {
    let clean = path.trim().trim_start_matches('/').replace('\\', "/");
    if clean.contains("..") {
        bail!("Path traversal attempt rejected: {}", path);
    }
    let mut parts: Vec<&str> = clean.split('/').filter(|s| !s.is_empty()).collect();
    if parts.is_empty() {
        bail!("Invalid empty path");
    }
    if parts[0] == "memory" && parts.len() > 1 {
        parts.remove(0);
    }
    let cat = parts[0];
    if cat.starts_with('.') || !cat.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-') {
        bail!("Invalid category '{}' in path: {}", cat, path);
    }
    Ok(parts.join("/"))
}

fn scan_dir_recursive(base: &Path, current: &Path, list: &mut Vec<MemoryFileNode>) {
    if let Ok(entries) = std::fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str())
                && (name.starts_with('.') || name == "search_index") {
                    continue;
                }
            if path.is_dir() {
                scan_dir_recursive(base, &path, list);
            } else if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md")
                && let Ok(rel) = path.strip_prefix(base) {
                    let rel_str = rel.to_string_lossy().replace('\\', "/");
                    let parts: Vec<&str> = rel_str.split('/').collect();
                    let category = if parts.len() > 1 { parts[0].to_string() } else { "uncategorized".to_string() };
                    let filename = parts.last().unwrap_or(&"note.md").to_string();
                    let meta = entry.metadata().ok();
                    let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                    let updated_at = meta
                        .and_then(|m| m.modified().ok())
                        .map(|t| {
                            let dt: chrono::DateTime<chrono::Utc> = t.into();
                            dt.to_rfc3339()
                        })
                        .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

                    let title = if let Ok(content) = std::fs::read_to_string(&path) {
                        content
                            .lines()
                            .find(|l| l.starts_with('#'))
                            .map(|l| l.trim_start_matches('#').trim().to_string())
                            .unwrap_or_else(|| filename.clone())
                    } else {
                        filename.clone()
                    };

                    list.push(MemoryFileNode {
                        path: rel_str,
                        category,
                        filename,
                        title,
                        size_bytes,
                        updated_at,
                    });
                }
        }
    }
}

#[cfg(test)]
mod tests;


