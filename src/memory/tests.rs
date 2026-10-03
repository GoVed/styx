#[cfg(test)]
mod tests {
    use super::super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_hierarchical_memory_creation_and_listing() {
        let base_path = std::env::temp_dir().join(format!("styx_mem_hier_{}", uuid::Uuid::new_v4()));
        let search_idx = Arc::new(search::MemorySearchIndex::new(&base_path.join("search_index")).unwrap());
        let memory = MemoryManager::new(base_path.clone(), search_idx);
        memory.ensure_directories().unwrap();

        let dict_path = memory
            .create_file("dictionary", "slang_terms.md", "# Slang Dictionary\n\nETA = Estimated Time of Arrival")
            .await
            .unwrap();
        assert_eq!(dict_path, "dictionary/slang_terms.md");

        let people_path = memory
            .create_file("people", "alice.md", "# Alice\n\nClose colleague, prefers concise professional updates")
            .await
            .unwrap();
        assert_eq!(people_path, "people/alice.md");

        let tree = memory.list_tree().unwrap();
        assert!(tree.iter().any(|f| f.path == "dictionary/slang_terms.md" && f.category == "dictionary"));
        assert!(tree.iter().any(|f| f.path == "people/alice.md" && f.category == "people"));

        let ctx = memory.get_entity_context("Alice").unwrap();
        assert!(ctx.contains("Close colleague, prefers concise professional updates"));

        let _ = std::fs::remove_dir_all(&base_path);
    }

    #[tokio::test]
    async fn test_is_channel_ignored() {
        let base_path = std::env::temp_dir().join(format!("styx_mem_test_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(base_path.join("core")).unwrap();
        std::fs::create_dir_all(base_path.join("skills")).unwrap();
        std::fs::create_dir_all(base_path.join("scratchpad")).unwrap();

        let profile_content = "## Message Preferences\n\
            - Ignore messages from: spam-marketing-channel\n\
            - Mute alerts from spam-bot-channel\n";
        std::fs::write(base_path.join("core/user_profile.md"), profile_content).unwrap();

        let search_idx = Arc::new(search::MemorySearchIndex::new(&base_path.join("search_index")).unwrap());
        let memory = MemoryManager::new(base_path.clone(), search_idx);

        assert!(memory.is_channel_ignored("spam-marketing-channel"));
        assert!(memory.is_channel_ignored("spam-bot-channel"));
        assert!(!memory.is_channel_ignored("friendly-contact-id"));
        assert!(!memory.is_channel_ignored(""));

        let _ = std::fs::remove_dir_all(&base_path);
    }

    #[test]
    fn test_sanitize_rel_path_strips_memory_prefix() {
        assert_eq!(
            sanitize_rel_path("memory/people/john.md").unwrap(),
            "people/john.md"
        );
        assert_eq!(
            sanitize_rel_path("people/john.md").unwrap(),
            "people/john.md"
        );
        assert_eq!(
            sanitize_rel_path("/memory/skills/test.md").unwrap(),
            "skills/test.md"
        );
    }

    #[tokio::test]
    async fn test_write_file_preserves_profile_sections() {
        let base_path = std::env::temp_dir().join(format!("styx_mem_pres_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(base_path.join("core")).unwrap();

        let initial_profile = "# User Profile\n\n## Identity\n- Name: Alex\n\n## Location & Routine\n- City: San Francisco\n";
        std::fs::write(base_path.join("core/user_profile.md"), initial_profile).unwrap();

        let search_idx = Arc::new(search::MemorySearchIndex::new(&base_path.join("search_index")).unwrap());
        let memory = MemoryManager::new(base_path.clone(), search_idx);

        memory
            .write_file(
                "core/user_profile.md",
                "## Communication Handle\n- Handle: TerminalUser\n",
                None,
            )
            .await
            .unwrap();

        let updated = memory.read_file("core/user_profile.md").unwrap();
        assert!(updated.contains("## Identity\n- Name: Alex"), "Identity section was lost!");
        assert!(updated.contains("## Location & Routine\n- City: San Francisco"), "Location section was lost!");
        assert!(updated.contains("## Communication Handle\n- Handle: TerminalUser"), "Handle section missing!");

        let _ = std::fs::remove_dir_all(&base_path);
    }

    #[test]
    fn test_semantic_embedding_cosine_similarity() {
        let vec1 = embeddings::SemanticEmbedder::fast_semantic_encode("Styx Personal AI Operating System harness in Rust");
        let vec2 = embeddings::SemanticEmbedder::fast_semantic_encode("Styx autonomous agent OS written in Rust");
        let vec3 = embeddings::SemanticEmbedder::fast_semantic_encode("Baking chocolate chip cookies recipe");

        let sim_related = vector::cosine_similarity(&vec1, &vec2);
        let sim_unrelated = vector::cosine_similarity(&vec1, &vec3);

        assert!(
            sim_related > sim_unrelated,
            "Related AI/Rust topics (sim: {}) must have higher similarity than unrelated cooking topics (sim: {})",
            sim_related,
            sim_unrelated
        );
    }

    #[tokio::test]
    async fn test_hybrid_search_and_image_indexing() {
        let base_path = std::env::temp_dir().join(format!("styx_mem_hybrid_{}", uuid::Uuid::new_v4()));
        let search_dir = base_path.join("search_index");
        let search_idx = Arc::new(search::MemorySearchIndex::new(&search_dir).unwrap());
        let memory = MemoryManager::new(base_path.clone(), search_idx.clone());
        memory.ensure_directories().unwrap();

        // 1. Create a markdown note
        memory
            .write_file("projects/styx_architecture.md", "# Styx Architecture\nAutonomous AI Harness with Tantivy and Vector search", None)
            .await
            .unwrap();

        // 2. Create an image and companion note in media/
        let media_dir = base_path.join("media");
        std::fs::create_dir_all(&media_dir).unwrap();
        let mut img = image::RgbImage::new(20, 20);
        for p in img.pixels_mut() {
            *p = image::Rgb([50, 100, 150]);
        }
        let img_path = media_dir.join("diagram.png");
        img.save(&img_path).unwrap();
        std::fs::write(media_dir.join("diagram.png.txt"), "System Architecture Diagram for Styx Harness").unwrap();

        // 3. Sync all files into search index
        search_idx.sync_all_files(&base_path).await.unwrap();

        // 4. Search text
        let results = memory.search("Styx Architecture", 5).unwrap();
        assert!(!results.is_empty());
        assert!(results.iter().any(|r| r.path.contains("styx_architecture.md")));

        // 5. Search image
        let img_results = memory.search("System Architecture Diagram", 5).unwrap();
        assert!(!img_results.is_empty());
        assert!(img_results.iter().any(|r| r.media_type == "image"));

        let _ = std::fs::remove_dir_all(&base_path);
    }

    #[tokio::test]
    async fn test_active_memory_context_fetching_for_group() {
        let base_path = std::env::temp_dir().join(format!("styx_mem_active_{}", uuid::Uuid::new_v4()));
        let search_dir = base_path.join("search_index");
        let search_idx = Arc::new(search::MemorySearchIndex::new(&search_dir).unwrap());
        let memory = MemoryManager::new(base_path.clone(), search_idx.clone());
        memory.ensure_directories().unwrap();

        // Create group memory file matching test scenario
        memory
            .write_file(
                "groups/tech_peers.md",
                "# Group: Tech Peers\n\n## Group Details\n- **Identifier**: `120363000000000001@g.us`\n- **Role**: Tech group with peers\n",
                None,
            )
            .await
            .unwrap();

        search_idx.sync_all_files(&base_path).await.unwrap();

        // User asks: "can you check tech peers if anyone used styx?"
        let active = memory.fetch_active_context("can you check tech peers if anyone used styx?", None);
        assert!(!active.is_empty(), "Active memory should find the entity!");
        let block = active.formatted_prompt.expect("Formatted prompt block must be generated");

        assert!(block.contains("groups/tech_peers.md"));
        assert!(block.contains("120363000000000001@g.us"));
        assert!(block.contains("Tech group with peers"));
        assert!(block.contains("OPERATING RULE"));

        let _ = std::fs::remove_dir_all(&base_path);
    }

    #[test]
    fn test_search_excerpt_utf8_char_boundary_safety() {
        let emoji_content = format!(
            "# Contact Profile\n\n- Phone: 1234567890\n- Description: Fresh {} healthy organic foods\n",
            "🥦".repeat(50)
        );
        let excerpt = search::MemorySearchIndex::extract_snippet(&emoji_content, "healthy", 80);
        assert!(!excerpt.is_empty());
        assert!(excerpt.contains("healthy"));
    }
}
