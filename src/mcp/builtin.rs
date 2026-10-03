use serde_json::json;

use super::policy::PolicyTier;
use super::types::McpTool;

pub fn get_builtin_tools() -> Vec<McpTool> {
    vec![
        McpTool {
            name: "read_memory".to_string(),
            description: "Read contents of a persistent memory, dictionary, contact, group, skill, or scratchpad file under /memory/ (e.g. dictionary/slang.md, people/alice.md, groups/project_alpha.md, core/user_profile.md, skills/*.md, scratchpad/*.md)".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative path under /memory/ (e.g. 'dictionary/slang.md', 'people/alice.md', or 'core/user_profile.md')"
                    }
                },
                "required": ["path"]
            }),
            server_id: None,
            policy: PolicyTier::Autonomous,
            risk_level: "LOW".to_string(),
        },
        McpTool {
            name: "search_memory".to_string(),
            description: "Fast hybrid semantic vector and full-text search across all markdown memory notes, user dictionaries, contact profiles, group files, modular skills, projects, and images using neural embeddings and Tantivy BM25".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search keyword or natural language query"
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of search results to return (default 5)"
                    }
                },
                "required": ["query"]
            }),
            server_id: None,
            policy: PolicyTier::Autonomous,
            risk_level: "LOW".to_string(),
        },
        McpTool {
            name: "write_memory".to_string(),
            description: "Autonomously write, update, or append notes to any persistent memory file under /memory/ (e.g. dictionary/*.md, people/*.md, groups/*.md, core/user_profile.md, skills/*.md, scratchpad/*.md). The agent has full authority to save, enrich, and maintain its memory.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Relative path under /memory/ (e.g. 'dictionary/slang.md', 'people/alice.md', 'groups/project_alpha.md', 'core/user_profile.md', or 'scratchpad/daily_log.md')"
                    },
                    "content": {
                        "type": "string",
                        "description": "Markdown content to write or append"
                    },
                    "section": {
                        "type": "string",
                        "description": "Optional section heading (e.g. 'Learned Vocabulary' or 'Action Items') to append under"
                    }
                },
                "required": ["path", "content"]
            }),
            server_id: None,
            policy: PolicyTier::Autonomous,
            risk_level: "LOW".to_string(),
        },
        McpTool {
            name: "exec_container_command".to_string(),
            description: "Execute a bash shell command or script inside the isolated Syndae container sandbox. Requires operator approval before execution.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command or script to execute inside the container"
                    },
                    "workdir": {
                        "type": "string",
                        "description": "Optional working directory inside the container (default: '/app')"
                    }
                },
                "required": ["command"]
            }),
            server_id: None,
            policy: PolicyTier::RequireApproval,
            risk_level: "CRITICAL".to_string(),
        },
        McpTool {
            name: "get_system_telemetry".to_string(),
            description: "Inspect host VM telemetry: CPU utilization, Memory RSS, Disk space, and Docker container statuses".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {}
            }),
            server_id: None,
            policy: PolicyTier::Autonomous,
            risk_level: "LOW".to_string(),
        },
        McpTool {
            name: "complete_onboarding".to_string(),
            description: "Mark user enrollment and initial onboarding setup as completed in Syndae. Call this tool once you have greeted the user, learned their initial preferences, and saved their profile.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "summary": {
                        "type": "string",
                        "description": "Brief summary of the enrolled user preferences and setup"
                    }
                }
            }),
            server_id: None,
            policy: PolicyTier::Autonomous,
            risk_level: "LOW".to_string(),
        },
        McpTool {
            name: "inspect_image".to_string(),
            description: "Visually inspect and analyze an image from an online HTTP URL, data URI, or local file path. Operator can select any vision model (e.g. moondream, qwen2-vl-2b) or custom vision endpoint.".to_string(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "url": {
                        "type": "string",
                        "description": "HTTP(S) URL, base64 data URI, or local file path of the image to inspect"
                    },
                    "question": {
                        "type": "string",
                        "description": "Optional specific question or prompt about the image. Defaults to a comprehensive visual description."
                    },
                    "model": {
                        "type": "string",
                        "description": "Optional vision model name (e.g. 'moondream', 'qwen2-vl-2b-local'). Defaults to active vision provider."
                    },
                    "endpoint": {
                        "type": "string",
                        "description": "Optional custom vision API base URL (e.g. 'http://localhost:11434/v1')."
                    }
                },
                "required": ["url"]
            }),
            server_id: None,
            policy: PolicyTier::Autonomous,
            risk_level: "LOW".to_string(),
        },
    ]
}
