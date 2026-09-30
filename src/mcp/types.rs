use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::policy::PolicyTier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub server_id: Option<String>,
    pub policy: PolicyTier,
    pub risk_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallOutput {
    pub tool_name: String,
    pub success: bool,
    pub content: String,
    pub is_error: bool,
}
