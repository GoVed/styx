use anyhow::Result;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub title: String,
    pub mode: String,
    pub created_at: String,
    pub updated_at: String,
}

impl Database {
    pub async fn create_session(&self, title: &str, mode: &str) -> Result<ChatSession> {
        let conn = self.conn.lock().await;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO sessions (id, title, mode, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, title, mode, now, now],
        )?;
        Ok(ChatSession {
            id,
            title: title.to_string(),
            mode: mode.to_string(),
            created_at: now.clone(),
            updated_at: now,
        })
    }

    pub async fn list_sessions(&self) -> Result<Vec<ChatSession>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT id, title, mode, created_at, updated_at FROM sessions ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(ChatSession {
                id: row.get(0)?,
                title: row.get(1)?,
                mode: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        let mut sessions = Vec::new();
        for s in rows {
            sessions.push(s?);
        }
        Ok(sessions)
    }

    pub async fn get_session(&self, id: &str) -> Result<Option<ChatSession>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT id, title, mode, created_at, updated_at FROM sessions WHERE id = ?1")?;
        let mut rows = stmt.query_map(params![id], |row| {
            Ok(ChatSession {
                id: row.get(0)?,
                title: row.get(1)?,
                mode: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    pub async fn delete_session(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM messages WHERE session_id = ?1", params![id])?;
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![id])?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn touch_session(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        Ok(())
    }

    pub async fn update_session_title_and_mode(&self, id: &str, title: &str, mode: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE sessions SET title = ?1, mode = ?2, updated_at = ?3 WHERE id = ?4",
            params![title, mode, now, id],
        )?;
        Ok(())
    }

    pub async fn find_session_for_tool(
        &self,
        protocol: &str,
        identifier: &str,
        alt_identifier: Option<&str>,
    ) -> Result<Option<ChatSession>> {
        let conn = self.conn.lock().await;
        let proto_tag = format!("[{}]", protocol.to_uppercase());
        let like_tag = format!("%{}%", proto_tag);
        let like_id = format!("%{}%", identifier);

        // 1. Primary lookup: protocol tag AND identifier in title
        let mut stmt = conn.prepare(
            "SELECT id, title, mode, created_at, updated_at FROM sessions 
             WHERE title LIKE ?1 AND title LIKE ?2 
             ORDER BY updated_at DESC"
        )?;
        let rows = stmt.query_map(params![like_tag, like_id], |row| {
            Ok(ChatSession {
                id: row.get(0)?,
                title: row.get(1)?,
                mode: row.get(2)?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        let mut matched: Vec<ChatSession> = Vec::new();
        for r in rows {
            matched.push(r?);
        }

        // 2. Fallback lookup: if no matches, try alt_identifier (e.g. sender name)
        if matched.is_empty()
            && let Some(alt) = alt_identifier {
                let trimmed = alt.trim();
                if !trimmed.is_empty() && trimmed != "Unknown" {
                    let like_alt = format!("%{}%", trimmed);
                    let mut stmt2 = conn.prepare(
                        "SELECT id, title, mode, created_at, updated_at FROM sessions 
                         WHERE title LIKE ?1 AND title LIKE ?2 
                         ORDER BY updated_at DESC"
                    )?;
                    let rows2 = stmt2.query_map(params![like_tag, like_alt], |row| {
                        Ok(ChatSession {
                            id: row.get(0)?,
                            title: row.get(1)?,
                            mode: row.get(2)?,
                            created_at: row.get(3)?,
                            updated_at: row.get(4)?,
                        })
                    })?;
                    for r in rows2 {
                        matched.push(r?);
                    }
                }
            }

        if matched.is_empty() {
            return Ok(None);
        }

        let primary = matched[0].clone();

        // Automatically consolidate any duplicate sessions for this contact/thread into primary
        if matched.len() > 1 {
            for dup in &matched[1..] {
                let _ = conn.execute(
                    "UPDATE messages SET session_id = ?1 WHERE session_id = ?2",
                    params![primary.id, dup.id],
                );
                let _ = conn.execute("DELETE FROM sessions WHERE id = ?1", params![dup.id]);
            }
        }

        Ok(Some(primary))
    }
}
