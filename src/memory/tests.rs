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

        // Create in dictionary/ and people/
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

        // Context lookup
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

        // Update display name without passing section
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
}
