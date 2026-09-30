pub mod auth;
pub mod messages;
pub mod models;
pub mod sessions;
pub mod tools;

#[cfg(test)]
mod tests;

#[allow(unused_imports)]
pub use auth::*;
#[allow(unused_imports)]
pub use messages::ChatMessage;
#[allow(unused_imports)]
pub use models::ModelConfigRecord;
#[allow(unused_imports)]
pub use sessions::ChatSession;
#[allow(unused_imports)]
pub use tools::{ApprovalTicketRecord, AuditEventRecord, McpServerRecord, ToolPolicyRecord};

use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn init(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).context("Failed to create db directory")?;
        }

        let conn = Connection::open(db_path).context("Failed to open sqlite database")?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;

            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                mode TEXT NOT NULL DEFAULT 'chat',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                thought TEXT,
                tool_calls TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS approval_tickets (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                tool_name TEXT NOT NULL,
                arguments TEXT NOT NULL,
                risk_level TEXT NOT NULL,
                status TEXT NOT NULL,
                modified_arguments TEXT,
                explanation TEXT,
                created_at TEXT NOT NULL,
                resolved_at TEXT
            );

            CREATE TABLE IF NOT EXISTS mcp_servers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                transport_type TEXT NOT NULL,
                command TEXT,
                args_json TEXT,
                env_json TEXT,
                socket_path TEXT,
                url TEXT,
                enabled INTEGER NOT NULL DEFAULT 1,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS tool_policies (
                tool_name TEXT PRIMARY KEY,
                policy TEXT NOT NULL DEFAULT 'REQUIRE_APPROVAL',
                custom_risk_level TEXT,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS audit_events (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                event_type TEXT NOT NULL,
                tool_name TEXT,
                payload_json TEXT,
                decision TEXT,
                duration_ms INTEGER,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS model_configs (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                provider TEXT NOT NULL,
                base_url TEXT,
                api_key TEXT,
                model_id TEXT NOT NULL,
                context_length INTEGER NOT NULL DEFAULT 16384,
                is_active INTEGER NOT NULL DEFAULT 0,
                extra_flags_json TEXT,
                created_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS system_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#,
        )?;

        // Ensure schema upgrades
        let _ = conn.execute("ALTER TABLE sessions ADD COLUMN mode TEXT NOT NULL DEFAULT 'chat'", []);
        let _ = conn.execute("ALTER TABLE messages ADD COLUMN thought TEXT", []);
        let _ = conn.execute("ALTER TABLE messages ADD COLUMN tool_calls TEXT", []);
        let _ = conn.execute("ALTER TABLE messages ADD COLUMN images TEXT", []);
        let _ = conn.execute("ALTER TABLE approval_tickets ADD COLUMN modified_arguments TEXT", []);
        let _ = conn.execute("ALTER TABLE approval_tickets ADD COLUMN explanation TEXT", []);
        let _ = conn.execute("ALTER TABLE tool_policies ADD COLUMN custom_risk_level TEXT", []);

        // Seed default model configs if empty
        let count: i64 = conn.query_row("SELECT COUNT(*) FROM model_configs", [], |r| r.get(0))?;
        if count == 0 {
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO model_configs (id, name, provider, base_url, api_key, model_id, context_length, is_active, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    Uuid::new_v4().to_string(),
                    "Local llama.cpp (Qwen 3.5 9B 128k Local GGUF)",
                    "docker_llamacpp",
                    "http://localhost:8080/v1",
                    "",
                    "/models/Qwen3.5-9B-UD-Q4_K_XL.gguf",
                    131072,
                    1,
                    now
                ],
            )?;

            conn.execute(
                "INSERT INTO model_configs (id, name, provider, base_url, api_key, model_id, context_length, is_active, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    Uuid::new_v4().to_string(),
                    "Local vLLM (Qwen 3.5 9B AWQ)",
                    "docker_vllm",
                    "http://localhost:8000/v1",
                    "",
                    "QuantTrio/Qwen3.5-9B-AWQ",
                    16384,
                    0,
                    now
                ],
            )?;

            conn.execute(
                "INSERT INTO model_configs (id, name, provider, base_url, api_key, model_id, context_length, is_active, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    Uuid::new_v4().to_string(),
                    "Claude 3.7 Sonnet (Anthropic)",
                    "anthropic",
                    "https://api.anthropic.com/v1",
                    "",
                    "claude-3-7-sonnet-20250219",
                    200000,
                    0,
                    now
                ],
            )?;
        }

        // Seed default tool policies
        let policy_count: i64 = conn.query_row("SELECT COUNT(*) FROM tool_policies", [], |r| r.get(0))?;
        if policy_count == 0 {
            let now = Utc::now().to_rfc3339();
            let default_policies = vec![
                ("read_memory", "AUTONOMOUS", "LOW"),
                ("search_memory", "AUTONOMOUS", "LOW"),
                ("write_memory", "AUTONOMOUS", "LOW"),
                ("exec_container_command", "AUTONOMOUS", "LOW"),
                ("get_system_telemetry", "AUTONOMOUS", "LOW"),
                ("inspect_image", "AUTONOMOUS", "LOW"),
                ("exec_bash", "REQUIRE_APPROVAL", "CRITICAL"),
                ("send_message", "REQUIRE_APPROVAL", "HIGH"),
                ("write_file", "REQUIRE_APPROVAL", "HIGH"),
            ];

            for (tool, policy, risk) in default_policies {
                conn.execute(
                    "INSERT OR REPLACE INTO tool_policies (tool_name, policy, custom_risk_level, updated_at)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![tool, policy, risk, now],
                )?;
            }
        }

        // Seed default reference MCP server
        let mcp_count: i64 = conn.query_row("SELECT COUNT(*) FROM mcp_servers", [], |r| r.get(0))?;
        if mcp_count == 0 {
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO mcp_servers (id, name, transport_type, command, args_json, env_json, socket_path, url, enabled, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 1, ?9)",
                params![
                    "builtin-reference-ops",
                    "Host System & Ops Micro-Daemon",
                    "stdio",
                    "python3",
                    "[\"./examples/reference_mcp_daemon.py\"]",
                    "{}",
                    "",
                    "",
                    now
                ],
            )?;
        }

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}
