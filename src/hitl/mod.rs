use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, oneshot, Mutex, RwLock};
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApprovalDecision {
    Approve,
    Reject { reason: Option<String> },
    ModifyPayload { new_arguments: Value },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalTicket {
    pub ticket_id: String,
    pub session_id: String,
    pub tool_name: String,
    pub arguments: Value,
    pub risk_level: String, // "LOW", "HIGH", "CRITICAL"
    pub explanation: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum HitlBroadcastEvent {
    TicketCreated { ticket: ApprovalTicket },
    TicketResolved {
        ticket_id: String,
        decision: String,
        modified_arguments: Option<Value>,
    },
}

#[derive(Clone)]
pub struct HitlGate {
    waiters: Arc<Mutex<HashMap<String, oneshot::Sender<ApprovalDecision>>>>,
    active_tickets: Arc<RwLock<HashMap<String, ApprovalTicket>>>,
    event_tx: broadcast::Sender<HitlBroadcastEvent>,
}

impl HitlGate {
    pub fn new() -> Self {
        let (event_tx, _) = broadcast::channel(100);
        Self {
            waiters: Arc::new(Mutex::new(HashMap::new())),
            active_tickets: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<HitlBroadcastEvent> {
        self.event_tx.subscribe()
    }

    pub async fn submit_for_approval(
        &self,
        ticket: ApprovalTicket,
    ) -> oneshot::Receiver<ApprovalDecision> {
        let (tx, rx) = oneshot::channel();
        let ticket_id = ticket.ticket_id.clone();

        {
            let mut tickets = self.active_tickets.write().await;
            tickets.insert(ticket_id.clone(), ticket.clone());
        }

        {
            let mut waiters = self.waiters.lock().await;
            waiters.insert(ticket_id.clone(), tx);
        }

        // Broadcast to WebSocket listeners
        let _ = self.event_tx.send(HitlBroadcastEvent::TicketCreated {
            ticket: ticket.clone(),
        });

        info!(
            "HITL Gate paused task! Ticket [{}] generated for mutating tool: '{}' (Risk: {})",
            ticket_id, ticket.tool_name, ticket.risk_level
        );

        rx
    }

    pub async fn resolve_approval(
        &self,
        ticket_id: &str,
        decision: ApprovalDecision,
    ) -> Result<()> {
        let sender = {
            let mut waiters = self.waiters.lock().await;
            waiters.remove(ticket_id)
        };

        let _ticket = {
            let mut tickets = self.active_tickets.write().await;
            tickets.remove(ticket_id)
        };

        if let Some(tx) = sender {
            let decision_str = match &decision {
                ApprovalDecision::Approve => "APPROVED".to_string(),
                ApprovalDecision::Reject { reason } => {
                    format!("REJECTED: {}", reason.as_deref().unwrap_or("User rejected"))
                }
                ApprovalDecision::ModifyPayload { .. } => "MODIFIED_AND_APPROVED".to_string(),
            };

            let mod_args = match &decision {
                ApprovalDecision::ModifyPayload { new_arguments } => Some(new_arguments.clone()),
                _ => None,
            };

            // Broadcast resolution
            let _ = self.event_tx.send(HitlBroadcastEvent::TicketResolved {
                ticket_id: ticket_id.to_string(),
                decision: decision_str,
                modified_arguments: mod_args,
            });

            if tx.send(decision).is_err() {
                warn!("HITL waiter receiver dropped before resolution for ticket {}", ticket_id);
            }

            info!("HITL ticket [{}] resolved successfully", ticket_id);
            Ok(())
        } else {
            bail!("No pending approval ticket found with ID: {}", ticket_id);
        }
    }

    pub async fn list_pending_tickets(&self) -> Vec<ApprovalTicket> {
        let tickets = self.active_tickets.read().await;
        tickets.values().cloned().collect()
    }

    #[allow(dead_code)]
    pub async fn get_ticket(&self, ticket_id: &str) -> Option<ApprovalTicket> {
        let tickets = self.active_tickets.read().await;
        tickets.get(ticket_id).cloned()
    }
}
