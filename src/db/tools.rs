use anyhow::Result;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalTicketRecord {
    pub id: String,
    pub session_id: String,
    pub tool_name: String,
    pub arguments: String,
    pub risk_level: String,
    pub status: String,
    pub modified_arguments: Option<String>,
    pub explanation: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerRecord {
    pub id: String,
    pub name: String,
    pub transport_type: String,
    pub command: Option<String>,
    pub args_json: Option<String>,
    pub env_json: Option<String>,
    pub socket_path: Option<String>,
    pub url: Option<String>,
    pub enabled: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolPolicyRecord {
    pub tool_name: String,
    pub policy: String, // "AUTONOMOUS", "REQUIRE_APPROVAL", "BLOCKED"
    pub custom_risk_level: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventRecord {
    pub id: String,
    pub session_id: Option<String>,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub payload_json: Option<String>,
    pub decision: Option<String>,
    pub duration_ms: Option<i64>,
    pub created_at: String,
}

impl Database {
    // Approval Tickets
    pub async fn create_approval_ticket(
        &self,
        id: &str,
        session_id: &str,
        tool_name: &str,
        arguments: &str,
        risk_level: &str,
        explanation: Option<&str>,
    ) -> Result<ApprovalTicketRecord> {
        let conn = self.conn.lock().await;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO approval_tickets (id, session_id, tool_name, arguments, risk_level, status, explanation, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, session_id, tool_name, arguments, risk_level, "pending", explanation, now],
        )?;
        Ok(ApprovalTicketRecord {
            id: id.to_string(),
            session_id: session_id.to_string(),
            tool_name: tool_name.to_string(),
            arguments: arguments.to_string(),
            risk_level: risk_level.to_string(),
            status: "pending".to_string(),
            modified_arguments: None,
            explanation: explanation.map(|s| s.to_string()),
            created_at: now,
            resolved_at: None,
        })
    }

    pub async fn resolve_approval_ticket(
        &self,
        id: &str,
        status: &str,
        modified_arguments: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().await;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "UPDATE approval_tickets SET status = ?1, modified_arguments = ?2, resolved_at = ?3 WHERE id = ?4",
            params![status, modified_arguments, now, id],
        )?;
        Ok(())
    }

    pub async fn list_approval_tickets(&self, limit: usize) -> Result<Vec<ApprovalTicketRecord>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, tool_name, arguments, risk_level, status, modified_arguments, explanation, created_at, resolved_at
             FROM approval_tickets ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(ApprovalTicketRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                tool_name: row.get(2)?,
                arguments: row.get(3)?,
                risk_level: row.get(4)?,
                status: row.get(5)?,
                modified_arguments: row.get(6)?,
                explanation: row.get(7)?,
                created_at: row.get(8)?,
                resolved_at: row.get(9)?,
            })
        })?;
        let mut tickets = Vec::new();
        for t in rows {
            tickets.push(t?);
        }
        Ok(tickets)
    }

    #[allow(dead_code)]
    pub async fn count_pending_approvals(&self) -> Result<i64> {
        let conn = self.conn.lock().await;
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM approval_tickets WHERE status = 'pending'",
            [],
            |r| r.get(0),
        )?;
        Ok(count)
    }

    // Tool Policies
    #[allow(dead_code)]
    pub async fn get_tool_policy(&self, tool_name: &str) -> Result<Option<ToolPolicyRecord>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT tool_name, policy, custom_risk_level, updated_at FROM tool_policies WHERE tool_name = ?1")?;
        let mut rows = stmt.query_map(params![tool_name], |row| {
            Ok(ToolPolicyRecord {
                tool_name: row.get(0)?,
                policy: row.get(1)?,
                custom_risk_level: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;
        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    pub async fn set_tool_policy(&self, tool_name: &str, policy: &str, risk: Option<&str>) -> Result<()> {
        let conn = self.conn.lock().await;
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO tool_policies (tool_name, policy, custom_risk_level, updated_at)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(tool_name) DO UPDATE SET policy = ?2, custom_risk_level = ?3, updated_at = ?4",
            params![tool_name, policy, risk, now],
        )?;
        Ok(())
    }

    pub async fn list_tool_policies(&self) -> Result<Vec<ToolPolicyRecord>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT tool_name, policy, custom_risk_level, updated_at FROM tool_policies ORDER BY tool_name ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(ToolPolicyRecord {
                tool_name: row.get(0)?,
                policy: row.get(1)?,
                custom_risk_level: row.get(2)?,
                updated_at: row.get(3)?,
            })
        })?;
        let mut policies = Vec::new();
        for p in rows {
            policies.push(p?);
        }
        Ok(policies)
    }

    // MCP Servers
    pub async fn list_mcp_servers(&self) -> Result<Vec<McpServerRecord>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, name, transport_type, command, args_json, env_json, socket_path, url, enabled, created_at
             FROM mcp_servers ORDER BY created_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            let enabled_int: i32 = row.get(8)?;
            Ok(McpServerRecord {
                id: row.get(0)?,
                name: row.get(1)?,
                transport_type: row.get(2)?,
                command: row.get(3)?,
                args_json: row.get(4)?,
                env_json: row.get(5)?,
                socket_path: row.get(6)?,
                url: row.get(7)?,
                enabled: enabled_int == 1,
                created_at: row.get(9)?,
            })
        })?;
        let mut servers = Vec::new();
        for s in rows {
            servers.push(s?);
        }
        Ok(servers)
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn add_mcp_server(
        &self,
        name: &str,
        transport_type: &str,
        command: Option<&str>,
        args_json: Option<&str>,
        env_json: Option<&str>,
        socket_path: Option<&str>,
        url: Option<&str>,
    ) -> Result<McpServerRecord> {
        let conn = self.conn.lock().await;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO mcp_servers (id, name, transport_type, command, args_json, env_json, socket_path, url, enabled, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9)",
            params![id, name, transport_type, command, args_json, env_json, socket_path, url, now],
        )?;
        Ok(McpServerRecord {
            id,
            name: name.to_string(),
            transport_type: transport_type.to_string(),
            command: command.map(|s| s.to_string()),
            args_json: args_json.map(|s| s.to_string()),
            env_json: env_json.map(|s| s.to_string()),
            socket_path: socket_path.map(|s| s.to_string()),
            url: url.map(|s| s.to_string()),
            enabled: true,
            created_at: now,
        })
    }

    pub async fn delete_mcp_server(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().await;
        conn.execute("DELETE FROM mcp_servers WHERE id = ?1", params![id])?;
        Ok(())
    }

    // Audit Events
    pub async fn record_audit_event(
        &self,
        session_id: Option<&str>,
        event_type: &str,
        tool_name: Option<&str>,
        payload_json: Option<&str>,
        decision: Option<&str>,
        duration_ms: Option<i64>,
    ) -> Result<AuditEventRecord> {
        let conn = self.conn.lock().await;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO audit_events (id, session_id, event_type, tool_name, payload_json, decision, duration_ms, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, session_id, event_type, tool_name, payload_json, decision, duration_ms, now],
        )?;
        Ok(AuditEventRecord {
            id,
            session_id: session_id.map(|s| s.to_string()),
            event_type: event_type.to_string(),
            tool_name: tool_name.map(|s| s.to_string()),
            payload_json: payload_json.map(|s| s.to_string()),
            decision: decision.map(|s| s.to_string()),
            duration_ms,
            created_at: now,
        })
    }

    pub async fn list_audit_events(&self, limit: usize) -> Result<Vec<AuditEventRecord>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, session_id, event_type, tool_name, payload_json, decision, duration_ms, created_at
             FROM audit_events ORDER BY created_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(AuditEventRecord {
                id: row.get(0)?,
                session_id: row.get(1)?,
                event_type: row.get(2)?,
                tool_name: row.get(3)?,
                payload_json: row.get(4)?,
                decision: row.get(5)?,
                duration_ms: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        let mut events = Vec::new();
        for e in rows {
            events.push(e?);
        }
        Ok(events)
    }
}
