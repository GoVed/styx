use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::hitl::ApprovalTicket;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentEvent {
    Queued {
        turn_id: String,
        queue_position: usize,
        total_queued: usize,
    },
    QueueStarted {
        turn_id: String,
    },
    Thought {
        content: String,
    },
    Token {
        content: String,
    },
    ToolCallStarted {
        id: String,
        tool_name: String,
        arguments: Value,
    },
    ToolPendingApproval {
        ticket: ApprovalTicket,
    },
    ToolApproved {
        ticket_id: String,
    },
    ToolRejected {
        ticket_id: String,
        reason: Option<String>,
    },
    ToolCompleted {
        id: String,
        tool_name: String,
        success: bool,
        output: String,
        duration_ms: u64,
    },
    Done {
        message_id: String,
    },
    OnboardingCompleted,
    Error {
        message: String,
    },
}
