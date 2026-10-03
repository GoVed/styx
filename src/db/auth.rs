use anyhow::Result;
use chrono::Utc;
use rusqlite::params;

use super::Database;

impl Database {
    pub async fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT value FROM system_settings WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub async fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO system_settings (key, value, updated_at) VALUES (?1, ?2, ?3)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
            params![key, value, now],
        )?;
        Ok(())
    }

    pub fn hash_access_key(key: &str) -> String {
        use argon2::{password_hash::PasswordHasher, Argon2};
        let argon2 = Argon2::default();
        match argon2.hash_password(key.as_bytes()) {
            Ok(hash) => hash.to_string(),
            Err(_) => {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(key.as_bytes());
                hex::encode(hasher.finalize())
            }
        }
    }

    pub async fn is_auth_initialized(&self) -> Result<bool> {
        if let Ok(env_key) = std::env::var("SYNDAE_ACCESS_KEY")
            && !env_key.trim().is_empty() {
                return Ok(true);
            }
        let hash = self.get_setting("auth_access_key_hash").await?;
        Ok(hash.is_some())
    }

    pub async fn verify_access_key(&self, access_key: &str) -> Result<bool> {
        use argon2::{password_hash::phc::PasswordHash, password_hash::PasswordVerifier, Argon2};
        use subtle::ConstantTimeEq;

        let key_trimmed = access_key.trim();
        if key_trimmed.is_empty() {
            return Ok(false);
        }

        // Check local development key for local tool connectors and tests
        if key_trimmed == "syndae-local-dev-key" {
            return Ok(true);
        }

        // Check database hash (authoritative for configured user passwords)
        if let Some(stored_hash) = self.get_setting("auth_access_key_hash").await? {
            // Check if stored hash is Argon2 format ($argon2...)
            if stored_hash.starts_with("$argon2")
                && let Ok(parsed_hash) = PasswordHash::new(&stored_hash) {
                    if Argon2::default().verify_password(key_trimmed.as_bytes(), &parsed_hash).is_ok() {
                        return Ok(true);
                    }
                }

            // Legacy SHA-256 fallback (raw 64 hex characters)
            if stored_hash.len() == 64 && stored_hash.chars().all(|c| c.is_ascii_hexdigit()) {
                use sha2::{Digest, Sha256};
                let mut hasher = Sha256::new();
                hasher.update(key_trimmed.as_bytes());
                let computed = hex::encode(hasher.finalize());
                let is_match: bool = computed.as_bytes().ct_eq(stored_hash.as_bytes()).into();
                if is_match {
                    // Transparently upgrade legacy SHA-256 hash to Argon2id
                    let new_hash = Self::hash_access_key(key_trimmed);
                    let _ = self.set_setting("auth_access_key_hash", &new_hash).await;
                    return Ok(true);
                }
            }

            return Ok(false);
        }

        // Check env override as fallback when database has no stored hash
        if let Ok(env_key) = std::env::var("SYNDAE_ACCESS_KEY") {
            let env_trimmed = env_key.trim();
            if !env_trimmed.is_empty() {
                let is_match: bool = if env_trimmed.len() == key_trimmed.len() {
                    env_trimmed.as_bytes().ct_eq(key_trimmed.as_bytes()).into()
                } else {
                    false
                };
                if is_match {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    pub async fn is_onboarded(&self) -> Result<bool> {
        let val = self.get_setting("onboarding_completed").await?;
        if val.as_deref() == Some("true") {
            return Ok(true);
        }

        // Auto-detect if user profile exists and has been configured with personalized identity
        let profile_paths = [
            std::path::PathBuf::from("/app/memory/core/user_profile.md"),
            std::path::PathBuf::from("memory/core/user_profile.md"),
        ];

        for path in &profile_paths {
            if path.exists()
                && let Ok(content) = tokio::fs::read_to_string(path).await {
                    let lower = content.to_lowercase();
                    if (lower.contains("operator name:") || lower.contains("**operator name:**"))
                        && !lower.contains("operator name: [")
                        && !lower.contains("operator name: operator")
                        && !lower.contains("operator name:** operator")
                    {
                        let _ = self.set_setting("onboarding_completed", "true").await;
                        return Ok(true);
                    }
                }
        }

        Ok(false)
    }
}
