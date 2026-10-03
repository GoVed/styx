use super::search::SearchResult;
use super::MemoryManager;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ActiveEntity {
    pub name: String,
    pub path: String,
    pub category: String,
    pub content: String,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ActiveMemoryContext {
    pub direct_entities: Vec<ActiveEntity>,
    pub semantic_matches: Vec<SearchResult>,
    pub formatted_prompt: Option<String>,
}

impl ActiveMemoryContext {
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.direct_entities.is_empty() && self.semantic_matches.is_empty()
    }
}

pub fn fetch_active_context(
    memory: &MemoryManager,
    user_prompt: &str,
    _images: Option<&[String]>,
    limit: usize,
) -> ActiveMemoryContext {
    let mut direct_entities = Vec::new();
    let mut seen_paths = std::collections::HashSet::new();

    // 1. Scan candidate entity phrases (1 to 4 words) from the prompt
    let candidates = extract_entity_candidates(user_prompt);
    for cand in &candidates {
        if let Some((path, cat, content)) = find_exact_entity(memory, cand)
            && seen_paths.insert(path.clone()) {
                direct_entities.push(ActiveEntity {
                    name: cand.clone(),
                    path,
                    category: cat,
                    content,
                });
            }
    }

    // 2. Perform fast hybrid semantic search across memory
    let search_results = memory.search(user_prompt, limit * 2).unwrap_or_default();
    let mut semantic_matches = Vec::new();

    for sr in search_results {
        // Skip core files and scratchpads already permanently loaded in system context
        if sr.category == "core" || sr.category == "scratchpad" {
            continue;
        }
        // Skip already matched direct entities
        if seen_paths.contains(&sr.path) {
            continue;
        }
        if sr.score >= 0.25 {
            seen_paths.insert(sr.path.clone());
            semantic_matches.push(sr);
            if semantic_matches.len() >= limit {
                break;
            }
        }
    }

    // 3. Format into a high-visibility working memory block
    let formatted_prompt = if direct_entities.is_empty() && semantic_matches.is_empty() {
        None
    } else {
        let mut block = String::new();
        block.push_str("\n\n=== ACTIVELY RETRIEVED RELEVANT MEMORY (FROM YOUR WORLD) ===\n");
        block.push_str("The runtime actively searched and found the following relevant memory entities and records in your private memory for this query:\n\n");

        for entity in &direct_entities {
            block.push_str(&format!(
                "[MATCHED ENTITY: {}] (Category: {})\n{}\n\n",
                entity.path, entity.category, entity.content.trim()
            ));
        }

        for match_item in &semantic_matches {
            let media_tag = if match_item.media_type == "image" { " [IMAGE ASSET]" } else { "" };
            block.push_str(&format!(
                "[RELEVANT MEMORY{}: {} | Relevance Score: {:.2}]\n{}\n\n",
                media_tag, match_item.path, match_item.score, match_item.snippet.trim()
            ));
        }

        block.push_str("OPERATING RULE: The user is likely asking about or referring to the entities and facts above. Consult and ground your response in this memory before considering external web searches or generic assumptions.\n");
        block.push_str("============================================================\n");
        Some(block)
    };

    ActiveMemoryContext {
        direct_entities,
        semantic_matches,
        formatted_prompt,
    }
}

fn extract_entity_candidates(prompt: &str) -> Vec<String> {
    let clean: String = prompt
        .chars()
        .map(|c| if c.is_alphanumeric() || c.is_whitespace() || c == '_' || c == '-' { c } else { ' ' })
        .collect();

    let words: Vec<&str> = clean.split_whitespace().collect();
    let mut candidates = Vec::new();
    let n = words.len();

    // 1-word, 2-word, 3-word, 4-word windows
    for window in 1..=4.min(n) {
        for start in 0..=(n - window) {
            let phrase = words[start..start + window].join(" ");
            let lower = phrase.to_lowercase();
            if lower.len() >= 3
                && !is_common_phrase(&lower)
                && !candidates.contains(&lower)
            {
                candidates.push(lower);
            }
        }
    }

    candidates
}

fn find_exact_entity(memory: &MemoryManager, candidate: &str) -> Option<(String, String, String)> {
    let clean = candidate.trim().to_lowercase().replace([' ', '@', '.', ':', '+', '-'], "_");
    let raw = candidate.trim().to_lowercase();

    for cat in ["groups", "people", "projects", "dictionary", "skills"] {
        for name in [&clean, &raw] {
            let rel_path = format!("{}/{}.md", cat, name);
            if let Ok(content) = memory.read_file(&rel_path) {
                return Some((rel_path, cat.to_string(), content));
            }
        }
    }
    None
}

fn is_common_phrase(s: &str) -> bool {
    matches!(
        s,
        "can you" | "check if" | "anyone used" | "what is" | "tell me" | "how to" | "if anyone" | "there is" | "used styx" | "is the"
    )
}
