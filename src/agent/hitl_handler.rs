use serde_json::Value;
use std::time::Duration;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::agent::types::AgentEvent;
use crate::db::Database;
use crate::hitl::{ApprovalDecision, ApprovalTicket, HitlGate};

pub async fn handle_hitl_approval(
    db: &Database,
    hitl: &HitlGate,
    session_id: &str,
    tool_name: &str,
    args_str: &str,
    parsed_args: &Value,
    risk_level: &str,
    event_tx: &mpsc::Sender<AgentEvent>,
) -> (Value, bool, Option<String>) {
    let ticket_id = Uuid::new_v4().to_string();
    let ticket = ApprovalTicket {
        ticket_id: ticket_id.clone(),
        session_id: session_id.to_string(),
        tool_name: tool_name.to_string(),
        arguments: parsed_args.clone(),
        risk_level: risk_level.to_string(),
        explanation: Some(format!("Agent requested state-mutating tool '{}'", tool_name)),
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    let _ = db
        .create_approval_ticket(&ticket_id, session_id, tool_name, args_str, risk_level, ticket.explanation.as_deref())
        .await;
    let _ = event_tx.send(AgentEvent::ToolPendingApproval { ticket: ticket.clone() }).await;

    let rx = hitl.submit_for_approval(ticket).await;
    let decision = match tokio::time::timeout(Duration::from_secs(120), rx).await {
        Ok(Ok(dec)) => dec,
        Ok(Err(_)) => ApprovalDecision::Reject { reason: Some("Approval channel canceled".to_string()) },
        Err(_) => {
            let reason = "Approval request timed out after 120s with no operator response.".to_string();
            let _ = hitl.resolve_approval(&ticket_id, ApprovalDecision::Reject { reason: Some(reason.clone()) }).await;
            ApprovalDecision::Reject { reason: Some(reason) }
        }
    };

    match decision {
        ApprovalDecision::Approve => {
            let _ = db.resolve_approval_ticket(&ticket_id, "approved", None).await;
            let _ = event_tx.send(AgentEvent::ToolApproved { ticket_id }).await;
            (parsed_args.clone(), true, None)
        }
        ApprovalDecision::ModifyPayload { new_arguments } => {
            let mod_str = serde_json::to_string(&new_arguments).unwrap_or_default();
            let _ = db.resolve_approval_ticket(&ticket_id, "modified", Some(&mod_str)).await;
            let _ = event_tx.send(AgentEvent::ToolApproved { ticket_id }).await;
            (new_arguments, true, None)
        }
        ApprovalDecision::Reject { reason } => {
            let _ = db.resolve_approval_ticket(&ticket_id, "rejected", reason.as_deref()).await;
            let _ = event_tx.send(AgentEvent::ToolRejected { ticket_id, reason: reason.clone() }).await;
            (parsed_args.clone(), false, reason)
        }
    }
}
