use anyhow::Result;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub thought: Option<String>,
    pub tool_calls: Option<String>,
    pub images: Option<String>,
    pub created_at: String,
}

impl Database {
    pub async fn add_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
        thought: Option<&str>,
        tool_calls: Option<&str>,
    ) -> Result<ChatMessage> {
        self.add_message_full(session_id, role, content, thought, tool_calls, None).await
    }

    pub async fn add_message_full(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
        thought: Option<&str>,
        tool_calls: Option<&str>,
        images: Option<&str>,
    ) -> Result<ChatMessage> {
        let conn = self.conn.lock().await;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO messages (id, session_id, role, content, thought, tool_calls, images, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, session_id, role, content, thought, tool_calls, images, now],
        )?;
        conn.execute(
            "UPDATE sessions SET updated_at = ?1 WHERE id = ?2",
            params![now, session_id],
        )?;
        Ok(ChatMessage {
            id,
            session_id: session_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            thought: thought.map(|s| s.to_string()),
            tool_calls: tool_calls.map(|s| s.to_string()),
            images: images.map(|s| s.to_string()),
            created_at: now,
        })
    }

    pub async fn list_messages(&self, session_id: &str) -> Result<Vec<ChatMessage>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, thought, tool_calls, images, created_at
             FROM messages WHERE session_id = ?1 ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map(params![session_id], |row| {
            Ok(ChatMessage {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                thought: row.get(4)?,
                tool_calls: row.get(5)?,
                images: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        let mut messages = Vec::new();
        for m in rows {
            messages.push(m?);
        }
        Ok(messages)
    }
}
