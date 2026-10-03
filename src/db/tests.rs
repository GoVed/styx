#[cfg(test)]
mod tests {
    use super::super::Database;

    #[tokio::test]
    async fn test_find_session_for_tool_and_consolidation() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_syndae_sessions_{}.db", uuid::Uuid::new_v4()));
        let db = Database::init(&db_path).expect("Failed to create test db");

        // 1. Create older session for WhatsApp contact Alice
        let s1 = db
            .create_session("[WHATSAPP] Alice (155500011122233@lid)", "mission")
            .await
            .unwrap();
        let _ = db
            .add_message(&s1.id, "user", "Message 1", None, None)
            .await
            .unwrap();

        // Allow timestamp advancement
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // 2. Create second duplicate session for Alice (simulating old behavior)
        let s2 = db
            .create_session("[WHATSAPP] Alice (155500011122233@lid)", "mission")
            .await
            .unwrap();
        let _ = db
            .add_message(&s2.id, "user", "Message 2", None, None)
            .await
            .unwrap();

        // Verify there are 2 sessions initially
        let initial_sessions = db.list_sessions().await.unwrap();
        let contact_sessions: Vec<_> = initial_sessions
            .iter()
            .filter(|s| s.title.contains("155500011122233@lid"))
            .collect();
        assert_eq!(contact_sessions.len(), 2);

        // 3. Call find_session_for_tool -> should return s2 (most recent) and consolidate s1 into s2
        let found = db
            .find_session_for_tool("whatsapp", "155500011122233@lid", Some("Alice"))
            .await
            .unwrap();

        assert!(found.is_some());
        let primary = found.unwrap();
        assert_eq!(primary.id, s2.id);

        // Verify that s1 was consolidated: only 1 session remains for Alice
        let updated_sessions = db.list_sessions().await.unwrap();
        let remaining_contact: Vec<_> = updated_sessions
            .iter()
            .filter(|s| s.title.contains("155500011122233@lid"))
            .collect();
        assert_eq!(remaining_contact.len(), 1);
        assert_eq!(remaining_contact[0].id, s2.id);

        // Verify both messages now belong to s2
        let messages = db.list_messages(&s2.id).await.unwrap();
        assert_eq!(messages.len(), 2);
        let contents: Vec<_> = messages.iter().map(|m| m.content.as_str()).collect();
        assert!(contents.contains(&"Message 1"));
        assert!(contents.contains(&"Message 2"));

        // Clean up temp file
        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn test_access_key_argon2_and_legacy_upgrade() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_syndae_auth_{}.db", uuid::Uuid::new_v4()));
        let db = Database::init(&db_path).expect("Failed to create test db");

        // 1. Initialized check should be false
        assert!(!db.is_auth_initialized().await.unwrap());

        // 2. Set modern Argon2 key
        let password = "super-secret-key-123";
        let argon2_hash = Database::hash_access_key(password);
        assert!(argon2_hash.starts_with("$argon2"));
        db.set_setting("auth_access_key_hash", &argon2_hash).await.unwrap();

        assert!(db.is_auth_initialized().await.unwrap());
        assert!(db.verify_access_key(password).await.unwrap());
        assert!(!db.verify_access_key("wrong-password").await.unwrap());

        // 3. Test legacy SHA-256 hash upgrade
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        let legacy_pass = "legacy-pass-456";
        hasher.update(legacy_pass.as_bytes());
        let legacy_sha256 = hex::encode(hasher.finalize());
        assert_eq!(legacy_sha256.len(), 64);

        db.set_setting("auth_access_key_hash", &legacy_sha256).await.unwrap();
        // Verification succeeds and auto-upgrades the hash
        assert!(db.verify_access_key(legacy_pass).await.unwrap());

        let upgraded_hash = db.get_setting("auth_access_key_hash").await.unwrap().unwrap();
        assert!(upgraded_hash.starts_with("$argon2"), "Hash should have been automatically upgraded to Argon2");
        assert!(db.verify_access_key(legacy_pass).await.unwrap());

        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn test_channel_trigger_policies_and_evaluation() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_syndae_channels_{}.db", uuid::Uuid::new_v4()));
        let db = Database::init(&db_path).expect("Failed to create test db");

        // 1. Direct chat defaults to Always Respond ('all')
        let (trigger, pol, _) = db
            .evaluate_channel_trigger(
                "15551112222@s.whatsapp.net",
                "whatsapp",
                Some("Alice"),
                false,
                "Hey how are you?",
                "",
                "Operator",
                false,
            )
            .await
            .unwrap();
        assert!(trigger, "Direct chat should trigger by default");
        assert_eq!(pol, "all");

        // 2. Group chat defaults to Mentions Only ('mentions')
        // 2a. Regular group chatter without mention -> Should NOT trigger
        let (trigger, pol, _) = db
            .evaluate_channel_trigger(
                "120363000000000001@g.us",
                "whatsapp",
                Some("Engineering Team"),
                true,
                "Good morning everyone! Check out this link.",
                "",
                "Operator",
                false,
            )
            .await
            .unwrap();
        assert!(!trigger, "Group chat without mention should NOT trigger");
        assert_eq!(pol, "mentions");

        // 2b. Group chat with mention of Syndae -> Should trigger
        let (trigger, pol, reason) = db
            .evaluate_channel_trigger(
                "120363000000000001@g.us",
                "whatsapp",
                Some("Engineering Team"),
                true,
                "Hey @Syndae, can you summarize our meeting?",
                "",
                "Operator",
                false,
            )
            .await
            .unwrap();
        assert!(trigger, "Group chat with mention of Syndae SHOULD trigger");
        assert_eq!(pol, "mentions");
        assert!(reason.contains("Mention"));

        // 2c. Group chat with mention of operator name ("Operator") -> Should trigger
        let (trigger, pol, _) = db
            .evaluate_channel_trigger(
                "120363000000000001@g.us",
                "whatsapp",
                Some("Engineering Team"),
                true,
                "Operator, what do you think about the architecture?",
                "",
                "Operator",
                false,
            )
            .await
            .unwrap();
        assert!(trigger, "Group chat with mention of operator name SHOULD trigger");
        assert_eq!(pol, "mentions");

        // 3. Explicitly Muted Group
        db.set_channel_policy(
            "120363000000000001@g.us",
            "whatsapp",
            Some("Engineering Team"),
            true,
            "muted",
            None,
        )
        .await
        .unwrap();

        let (trigger, pol, _) = db
            .evaluate_channel_trigger(
                "120363000000000001@g.us",
                "whatsapp",
                Some("Engineering Team"),
                true,
                "@Syndae please help!",
                "",
                "Operator",
                false,
            )
            .await
            .unwrap();
        assert!(!trigger, "Muted group must NEVER trigger even if mentioned");
        assert_eq!(pol, "muted");

        // 4. Explicitly Set Group to Always Respond ('all')
        db.set_channel_policy(
            "120363000000000001@g.us",
            "whatsapp",
            Some("Engineering Team"),
            true,
            "all",
            None,
        )
        .await
        .unwrap();

        let (trigger, pol, _) = db
            .evaluate_channel_trigger(
                "120363000000000001@g.us",
                "whatsapp",
                Some("Engineering Team"),
                true,
                "Just general chat",
                "",
                "Operator",
                false,
            )
            .await
            .unwrap();
        assert!(trigger, "Group set to 'all' should trigger on any message");
        assert_eq!(pol, "all");

        // 5. Memory-level ignore override
        let (trigger, pol, _) = db
            .evaluate_channel_trigger(
                "120363000000000001@g.us",
                "whatsapp",
                Some("Engineering Team"),
                true,
                "Hello",
                "",
                "Operator",
                true, // memory ignored
            )
            .await
            .unwrap();
        assert!(!trigger, "Memory ignored channel must not trigger");
        assert_eq!(pol, "muted");

        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn test_change_password_and_hash_invalidation() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_syndae_change_pass_{}.db", uuid::Uuid::new_v4()));
        let db = Database::init(&db_path).expect("Failed to create test db");

        // Initial password setup
        let initial_pass = "old-secret-password-1";
        let initial_hash = Database::hash_access_key(initial_pass);
        db.set_setting("auth_access_key_hash", &initial_hash).await.unwrap();

        assert!(db.verify_access_key(initial_pass).await.unwrap());
        assert!(!db.verify_access_key("wrong-password").await.unwrap());

        // Change password
        let new_pass = "brand-new-master-key-2026";
        let new_hash = Database::hash_access_key(new_pass);
        db.set_setting("auth_access_key_hash", &new_hash).await.unwrap();

        // New password must verify, old password must immediately fail
        assert!(db.verify_access_key(new_pass).await.unwrap());
        assert!(!db.verify_access_key(initial_pass).await.unwrap());

        let _ = std::fs::remove_file(db_path);
    }
}
