use anyhow::Result;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfigRecord {
    pub id: String,
    pub name: String,
    pub provider: String, // "docker_vllm", "docker_llamacpp", "docker_ollama", "openai", "anthropic", "gemini"
    pub base_url: Option<String>,
    pub api_key: Option<String>,
    pub model_id: String,
    pub context_length: i64,
    pub is_active: bool,
    pub extra_flags_json: Option<String>,
    pub created_at: String,
}

impl Database {
    pub async fn list_model_configs(&self) -> Result<Vec<ModelConfigRecord>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, provider, base_url, api_key, model_id, context_length, is_active, extra_flags_json, created_at
             FROM model_configs ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let active_int: i32 = row.get(7)?;
            Ok(ModelConfigRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                provider: row.get(2)?,
                base_url: row.get(3)?,
                api_key: row.get(4)?,
                model_id: row.get(5)?,
                context_length: row.get(6)?,
                is_active: active_int == 1,
                extra_flags_json: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        let mut configs = Vec::new();
        for c in rows {
            configs.push(c?);
        }
        Ok(configs)
    }

    pub async fn get_active_model_config(&self) -> Result<Option<ModelConfigRecord>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, provider, base_url, api_key, model_id, context_length, is_active, extra_flags_json, created_at
             FROM model_configs WHERE is_active = 1 LIMIT 1",
        )?;
        let mut rows = stmt.query_map([], |row| {
            let active_int: i32 = row.get(7)?;
            Ok(ModelConfigRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                provider: row.get(2)?,
                base_url: row.get(3)?,
                api_key: row.get(4)?,
                model_id: row.get(5)?,
                context_length: row.get(6)?,
                is_active: active_int == 1,
                extra_flags_json: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?;
        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    pub async fn set_active_model(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute("UPDATE model_configs SET is_active = 0", [])?;
        conn.execute(
            "UPDATE model_configs SET is_active = 1 WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub async fn get_active_vision_model_config(&self) -> Result<Option<ModelConfigRecord>> {
        let configs = self.list_model_configs().await?;
        for c in &configs {
            if let Some(ref flags) = c.extra_flags_json {
                if flags.contains("\"role\":\"vision\"") || flags.contains("\"role\": \"vision\"") {
                    return Ok(Some(c.clone()));
                }
            }
        }
        for c in &configs {
            let lower_id = c.model_id.to_lowercase();
            let lower_name = c.name.to_lowercase();
            if lower_id.contains("moondream")
                || lower_id.contains("qwen2-vl")
                || lower_id.contains("qwen2.5-vl")
                || lower_name.contains("vision sidecar")
            {
                return Ok(Some(c.clone()));
            }
        }
        Ok(None)
    }

    #[allow(dead_code)]
    pub async fn set_active_vision_model(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT id, extra_flags_json FROM model_configs WHERE extra_flags_json LIKE '%vision%'")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)))?;
        let items: Vec<(String, Option<String>)> = rows.filter_map(|r| r.ok()).collect();
        for (row_id, flags_opt) in items {
            if let Some(flags) = flags_opt {
                let cleaned = flags
                    .replace("\"role\":\"vision\",", "")
                    .replace(",\"role\":\"vision\"", "")
                    .replace("\"role\":\"vision\"", "");
                let _ = conn.execute(
                    "UPDATE model_configs SET extra_flags_json = ?1 WHERE id = ?2",
                    params![cleaned, row_id],
                );
            }
        }
        let flags = serde_json::json!({"role": "vision"}).to_string();
        conn.execute(
            "UPDATE model_configs SET extra_flags_json = ?1 WHERE id = ?2",
            params![flags, id],
        )?;
        Ok(())
    }

    pub async fn get_active_translation_model_config(&self) -> Result<Option<ModelConfigRecord>> {
        let configs = self.list_model_configs().await?;
        for c in &configs {
            if let Some(ref flags) = c.extra_flags_json {
                if flags.contains("\"role\":\"translation\"") || flags.contains("\"role\": \"translation\"") {
                    return Ok(Some(c.clone()));
                }
            }
        }
        for c in &configs {
            let lower_id = c.model_id.to_lowercase();
            let lower_name = c.name.to_lowercase();
            if lower_id.contains("gemma2:2b")
                || lower_id.contains("sarvam")
                || lower_id.contains("indic")
                || lower_name.contains("translation")
                || lower_name.contains("gujarati")
            {
                return Ok(Some(c.clone()));
            }
        }
        Ok(None)
    }

    pub async fn set_active_translation_model(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT id, extra_flags_json FROM model_configs WHERE extra_flags_json LIKE '%translation%'")?;
        let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?)))?;
        let items: Vec<(String, Option<String>)> = rows.filter_map(|r| r.ok()).collect();
        for (row_id, flags_opt) in items {
            if let Some(flags) = flags_opt {
                let cleaned = flags
                    .replace("\"role\":\"translation\",", "")
                    .replace(",\"role\":\"translation\"", "")
                    .replace("\"role\":\"translation\"", "");
                let _ = conn.execute(
                    "UPDATE model_configs SET extra_flags_json = ?1 WHERE id = ?2",
                    params![cleaned, row_id],
                );
            }
        }
        let flags = serde_json::json!({"role": "translation"}).to_string();
        conn.execute(
            "UPDATE model_configs SET extra_flags_json = ?1 WHERE id = ?2",
            params![flags, id],
        )?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_model_config(
        &self,
        name: &str,
        provider: &str,
        base_url: Option<&str>,
        api_key: Option<&str>,
        model_id: &str,
        context_length: i64,
        extra_flags_json: Option<&str>,
    ) -> Result<ModelConfigRecord> {
        let conn = self.conn.lock().await;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO model_configs (id, name, provider, base_url, api_key, model_id, context_length, is_active, extra_flags_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, ?8, ?9)",
            params![id, name, provider, base_url, api_key, model_id, context_length, extra_flags_json, now],
        )?;
        Ok(ModelConfigRecord {
            id,
            name: name.to_string(),
            provider: provider.to_string(),
            base_url: base_url.map(|s| s.to_string()),
            api_key: api_key.map(|s| s.to_string()),
            model_id: model_id.to_string(),
            context_length,
            is_active: false,
            extra_flags_json: extra_flags_json.map(|s| s.to_string()),
            created_at: now,
        })
    }

    pub async fn update_model_config(
        &self,
        id: &str,
        name: &str,
        model_id: &str,
        context_length: i64,
    ) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE model_configs SET name = ?1, model_id = ?2, context_length = ?3 WHERE id = ?4",
            params![name, model_id, context_length, id],
        )?;
        Ok(())
    }

    pub async fn update_model_config_flags(
        &self,
        id: &str,
        extra_flags_json: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute(
            "UPDATE model_configs SET extra_flags_json = ?1 WHERE id = ?2",
            params![extra_flags_json, id],
        )?;
        Ok(())
    }

    pub async fn delete_model_config(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM model_configs WHERE id = ?1", params![id])?;
        Ok(())
    }
}
