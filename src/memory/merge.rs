//! Intelligent Markdown Section Parser and Merging Engine
//!
//! Prevents memory loss by performing section-aware updates instead of
//! destructive full-file overwrites.

/// Upserts a section into markdown text.
/// If `## <section_name>` exists, replaces its content up to the next heading (`#` or `##`).
/// If not, appends `\n\n## <section_name>\n<content>\n`.
pub fn upsert_markdown_section(existing: &str, section_title: &str, content: &str) -> String {
    let clean_title = section_title.trim().trim_start_matches('#').trim();
    if clean_title.is_empty() {
        return existing.to_string();
    }

    let existing_trimmed = existing.trim();
    if existing_trimmed.is_empty() {
        return format!("## {}\n{}\n", clean_title, content.trim());
    }

    let lines: Vec<&str> = existing.lines().collect();
    let target_lower = clean_title.to_lowercase();

    // Look for matching heading: ## <title> or ### <title>
    let mut section_start_idx: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let header_text = trimmed.trim_start_matches('#').trim().to_lowercase();
            if header_text == target_lower {
                section_start_idx = Some(i);
                break;
            }
        }
    }

    if let Some(start_idx) = section_start_idx {
        // Section exists: find where it ends (next heading of equal or higher level: # or ##)
        let mut section_end_idx = lines.len();
        for (i, line) in lines.iter().enumerate().skip(start_idx + 1) {
            let trimmed = line.trim();
            if trimmed.starts_with("# ") || trimmed.starts_with("## ") {
                section_end_idx = i;
                break;
            }
        }

        let mut output = Vec::new();
        // Keep everything up to the section header
        for line in lines.iter().take(start_idx) {
            output.push((*line).to_string());
        }

        // Add section header and new content
        output.push(format!("## {}", clean_title));
        output.push(content.trim().to_string());

        // Add blank line before next section if there are remaining lines
        if section_end_idx < lines.len() {
            output.push(String::new());
        }

        // Keep everything after the section
        for line in lines.iter().skip(section_end_idx) {
            output.push((*line).to_string());
        }

        output.join("\n").trim().to_string() + "\n"
    } else {
        // Section does not exist: append to the end
        format!("{}\n\n## {}\n{}\n", existing_trimmed, clean_title, content.trim())
    }
}

/// Merges incoming markdown into an existing document without losing existing sections.
pub fn merge_markdown_documents(existing: &str, incoming: &str, target_section: Option<&str>) -> String {
    let incoming_trimmed = incoming.trim();
    if incoming_trimmed.is_empty() {
        return existing.to_string();
    }

    let existing_trimmed = existing.trim();
    if existing_trimmed.is_empty() {
        if let Some(sec) = target_section {
            return format!("## {}\n{}\n", sec.trim().trim_start_matches('#').trim(), incoming_trimmed);
        }
        return incoming.to_string();
    }

    // If an explicit section is specified, upsert into that section
    if let Some(sec) = target_section {
        return upsert_markdown_section(existing, sec, incoming_trimmed);
    }

    // Check if incoming text contains one or more `## ` section headers
    let incoming_lines: Vec<&str> = incoming_trimmed.lines().collect();
    let mut sections: Vec<(String, Vec<String>)> = Vec::new();
    let mut current_sec: Option<String> = None;
    let mut current_body: Vec<String> = Vec::new();
    let mut leading_lines: Vec<String> = Vec::new();

    for line in incoming_lines {
        let trimmed = line.trim();
        if trimmed.starts_with("## ") {
            if let Some(sec_name) = current_sec.take() {
                sections.push((sec_name, current_body));
                current_body = Vec::new();
            }
            let title = trimmed.trim_start_matches('#').trim().to_string();
            current_sec = Some(title);
        } else if current_sec.is_some() {
            current_body.push(line.to_string());
        } else {
            leading_lines.push(line.to_string());
        }
    }

    if let Some(sec_name) = current_sec {
        sections.push((sec_name, current_body));
    }

    if !sections.is_empty() {
        let mut current_doc = existing.to_string();
        for (sec_title, body_lines) in sections {
            let body_str = body_lines.join("\n");
            current_doc = upsert_markdown_section(&current_doc, &sec_title, &body_str);
        }
        current_doc
    } else if incoming_trimmed.starts_with("# ") {
        // Incoming is a full document with `# Title`.
        // If existing has sections not present in incoming, preserve them at the end.
        let mut merged = incoming_trimmed.to_string();
        // Parse existing sections
        let existing_lines: Vec<&str> = existing_trimmed.lines().collect();
        let mut curr_exist_sec: Option<String> = None;
        let mut curr_exist_body: Vec<String> = Vec::new();

        for line in existing_lines {
            let trimmed = line.trim();
            if trimmed.starts_with("## ") {
                if let Some(sec) = curr_exist_sec.take() {
                    let body = curr_exist_body.join("\n");
                    if !merged.to_lowercase().contains(&format!("## {}", sec.to_lowercase())) {
                        merged = upsert_markdown_section(&merged, &sec, &body);
                    }
                    curr_exist_body = Vec::new();
                }
                curr_exist_sec = Some(trimmed.trim_start_matches('#').trim().to_string());
            } else if curr_exist_sec.is_some() {
                curr_exist_body.push(line.to_string());
            }
        }
        if let Some(sec) = curr_exist_sec {
            let body = curr_exist_body.join("\n");
            if !merged.to_lowercase().contains(&format!("## {}", sec.to_lowercase())) {
                merged = upsert_markdown_section(&merged, &sec, &body);
            }
        }
        merged + "\n"
    } else {
        // Snippet without header: append under "Additional Notes" or update if already there
        upsert_markdown_section(existing, "Additional Notes", incoming_trimmed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upsert_new_section() {
        let base = "# User Profile\n\n## Identity\n- Name: Alex\n";
        let res = upsert_markdown_section(base, "Location", "- City: San Francisco");
        assert!(res.contains("## Identity\n- Name: Alex"));
        assert!(res.contains("## Location\n- City: San Francisco"));
    }

    #[test]
    fn test_upsert_existing_section_replaces_content() {
        let base = "# User Profile\n\n## Identity\n- Name: Bob\n\n## Location\n- City: Seattle\n";
        let res = upsert_markdown_section(base, "Identity", "- Name: Alex\n- Role: Architect");
        assert!(res.contains("- Name: Alex"));
        assert!(res.contains("- Role: Architect"));
        assert!(!res.contains("Bob"));
        assert!(res.contains("## Location\n- City: Seattle"));
    }

    #[test]
    fn test_merge_preserves_unrelated_sections() {
        let base = "# User Profile\n\n## Identity\n- Name: Alex\n\n## Location\n- City: San Francisco\n";
        let incoming = "## Preferred Shell\n- Shell: zsh\n";
        let res = merge_markdown_documents(base, incoming, None);
        assert!(res.contains("## Identity\n- Name: Alex"));
        assert!(res.contains("## Location\n- City: San Francisco"));
        assert!(res.contains("## Preferred Shell\n- Shell: zsh"));
    }

    #[test]
    fn test_merge_snippet_appends_to_additional_notes() {
        let base = "# User Profile\n\n## Identity\n- Name: Alex\n";
        let incoming = "- Prefers dark mode";
        let res = merge_markdown_documents(base, incoming, None);
        assert!(res.contains("## Identity\n- Name: Alex"));
        assert!(res.contains("## Additional Notes\n- Prefers dark mode"));
    }
}
