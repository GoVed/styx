#[cfg(test)]
mod tests {
    use super::super::Database;

    #[tokio::test]
    async fn test_find_session_for_tool_and_consolidation() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("test_styx_sessions_{}.db", uuid::Uuid::new_v4()));
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
        let db_path = temp_dir.join(format!("test_styx_auth_{}.db", uuid::Uuid::new_v4()));
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
}
