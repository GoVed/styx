use std::path::Path;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ImageMemoryItem {
    pub rel_path: String,
    pub category: String,
    pub filename: String,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub size_bytes: u64,
    pub description: String,
}

pub fn is_image_extension(ext: &str) -> bool {
    matches!(
        ext.to_lowercase().as_str(),
        "png" | "jpg" | "jpeg" | "webp" | "gif" | "bmp"
    )
}

pub fn inspect_image_file(base_dir: &Path, full_path: &Path) -> Option<ImageMemoryItem> {
    let ext = full_path.extension()?.to_str()?;
    if !is_image_extension(ext) {
        return None;
    }

    let rel = full_path.strip_prefix(base_dir).ok()?;
    let rel_str = rel.to_string_lossy().replace('\\', "/");
    let parts: Vec<&str> = rel_str.split('/').collect();
    let category = if parts.len() > 1 {
        parts[0].to_string()
    } else {
        "media".to_string()
    };

    let filename = full_path.file_name()?.to_str()?.to_string();
    let stem = full_path.file_stem()?.to_str()?.replace(['_', '-'], " ");
    let title = format!("Image: {}", capitalize_words(&stem));

    let (width, height) = match image::image_dimensions(full_path) {
        Ok((w, h)) => (w, h),
        Err(_) => (0, 0),
    };

    let meta = full_path.metadata().ok();
    let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);

    // Look for companion description files e.g. "photo.png.json" or "photo.md"
    let mut description = format!(
        "Image asset '{}' in category '{}'. Format: {}, Dimensions: {}x{}, Size: {} bytes.",
        filename, category, ext.to_uppercase(), width, height, size_bytes
    );

    let companion_txt = full_path.with_extension("txt");
    let companion_json = full_path.with_extension("json");
    let companion_md = full_path.with_extension("md");

    for candidate in [companion_txt, companion_json, companion_md] {
        if candidate.exists()
            && let Ok(text) = std::fs::read_to_string(&candidate) {
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    description.push_str(" Context & Notes: ");
                    description.push_str(trimmed);
                    break;
                }
            }
    }

    Some(ImageMemoryItem {
        rel_path: rel_str,
        category,
        filename,
        title,
        width,
        height,
        format: ext.to_uppercase(),
        size_bytes,
        description,
    })
}

pub fn scan_image_files(base: &Path, current: &Path, list: &mut Vec<ImageMemoryItem>) {
    if let Ok(entries) = std::fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str())
                && (name.starts_with('.') || name == "search_index") {
                    continue;
                }

            if path.is_dir() {
                scan_image_files(base, &path, list);
            } else if path.is_file()
                && let Some(item) = inspect_image_file(base, &path) {
                    list.push(item);
                }
        }
    }
}

pub fn walk_markdown_paths(
    base: &Path,
    current: &Path,
    files: &mut Vec<(String, String, String, String)>,
) {
    if let Ok(entries) = std::fs::read_dir(current) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|s| s.to_str())
                && (name.starts_with('.') || name == "search_index") {
                    continue;
                }
            if path.is_dir() {
                walk_markdown_paths(base, &path, files);
            } else if path.is_file()
                && path.extension().and_then(|s| s.to_str()) == Some("md")
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

fn capitalize_words(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}
