use std::collections::{HashMap, VecDeque};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex, Notify};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use tracing::{error, info};

use crate::agent::{AgentEvent, AgentRunner};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedTurnInfo {
    pub turn_id: String,
    pub session_id: String,
    pub mode: String,
    pub source: String,
    pub queued_at: DateTime<Utc>,
    pub position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTurnInfo {
    pub turn_id: String,
    pub session_id: String,
    pub mode: String,
    pub source: String,
    pub started_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatusResponse {
    pub max_concurrent_turns: usize,
    pub active_count: usize,
    pub queued_count: usize,
    pub active_turns: Vec<ActiveTurnInfo>,
    pub queued_turns: Vec<QueuedTurnInfo>,
}

pub struct QueuedJob {
    pub turn_id: String,
    pub session_id: String,
    pub prompt: String,
    pub mode: String,
    pub source: String,
    pub images: Option<Vec<String>>,
    pub queued_at: DateTime<Utc>,
    pub event_tx: mpsc::Sender<AgentEvent>,
}

pub struct TurnQueue {
    max_concurrent: AtomicUsize,
    queue: Mutex<VecDeque<QueuedJob>>,
    active: Mutex<HashMap<String, ActiveTurnInfo>>, // turn_id -> info
    agent: Arc<AgentRunner>,
    notify: Arc<Notify>,
}

impl TurnQueue {
    pub fn new(agent: Arc<AgentRunner>, max_concurrent: usize) -> Arc<Self> {
        let queue = Arc::new(Self {
            max_concurrent: AtomicUsize::new(max_concurrent.max(1)),
            queue: Mutex::new(VecDeque::new()),
            active: Mutex::new(HashMap::new()),
            agent,
            notify: Arc::new(Notify::new()),
        });

        let q_clone = Arc::clone(&queue);
        tokio::spawn(async move {
            q_clone.worker_loop().await;
        });

        queue
    }

    pub fn max_concurrent(&self) -> usize {
        self.max_concurrent.load(Ordering::Relaxed)
    }

    pub fn set_max_concurrent(&self, limit: usize) {
        let val = limit.max(1);
        self.max_concurrent.store(val, Ordering::Relaxed);
        info!("Updated max concurrent turns limit to {}", val);
        self.notify.notify_one();
    }

    pub async fn submit_turn(
        self: &Arc<Self>,
        session_id: &str,
        prompt: &str,
        mode: &str,
        source: &str,
        event_tx: mpsc::Sender<AgentEvent>,
    ) -> (String, usize, bool) {
        self.submit_turn_with_images(session_id, prompt, mode, source, None, event_tx).await
    }

    pub async fn submit_turn_with_images(
        self: &Arc<Self>,
        session_id: &str,
        prompt: &str,
        mode: &str,
        source: &str,
        images: Option<Vec<String>>,
        event_tx: mpsc::Sender<AgentEvent>,
    ) -> (String, usize, bool) {
        let turn_id = Uuid::new_v4().to_string();
        let mut queue = self.queue.lock().await;
        let active = self.active.lock().await;
        let max = self.max_concurrent.load(Ordering::Relaxed);

        // Prune stale tool jobs (> 300s old)
        let now = Utc::now();
        queue.retain(|j| {
            if j.source.starts_with("tool_") && (now - j.queued_at).num_seconds() > 300 {
                info!("Pruned stale queued tool turn {} for session {}", j.turn_id, j.session_id);
                false
            } else {
                true
            }
        });

        // Debounce background tool triggers: keep at most 2 queued tool jobs per session
        if source.starts_with("tool_") {
            let session_tool_jobs = queue
                .iter()
                .filter(|j| j.session_id == session_id && j.source.starts_with("tool_"))
                .count();
            if session_tool_jobs >= 2 {
                info!("Debouncing inbound tool turn for session {}: dropping surplus queued trigger", session_id);
                let _ = event_tx
                    .send(AgentEvent::Error {
                        message: "Debounced: session has pending tool turns".to_string(),
                    })
                    .await;
                return (turn_id, 0, false);
            }
        }

        let is_immediate = active.len() < max && queue.is_empty();
        let job = QueuedJob {
            turn_id: turn_id.clone(),
            session_id: session_id.to_string(),
            prompt: prompt.to_string(),
            mode: mode.to_string(),
            source: source.to_string(),
            images,
            queued_at: now,
            event_tx,
        };

        let position = if is_immediate {
            queue.push_back(job);
            0
        } else if source == "user" {
            // User prompts jump ahead of background tool turns
            let insert_idx = queue.iter().position(|j| j.source != "user").unwrap_or(queue.len());
            queue.insert(insert_idx, job);
            insert_idx + 1
        } else {
            queue.push_back(job);
            queue.len()
        };

        if !is_immediate {
            info!(
                "Turn {} ({}) for session {} queued at position {}/{} (active: {}, max: {})",
                turn_id, source, session_id, position, queue.len(), active.len(), max
            );

            // Re-broadcast updated positions to all queued jobs
            for (idx, waiting) in queue.iter().enumerate() {
                let _ = waiting
                    .event_tx
                    .send(AgentEvent::Queued {
                        turn_id: waiting.turn_id.clone(),
                        queue_position: idx + 1,
                        total_queued: queue.len(),
                    })
                    .await;
            }
        }

        drop(active);
        drop(queue);
        self.notify.notify_one();

        (turn_id, position, is_immediate)
    }

    pub async fn clear_queue(&self) -> usize {
        let mut queue = self.queue.lock().await;
        let count = queue.len();
        for job in queue.drain(..) {
            let _ = job
                .event_tx
                .send(AgentEvent::Error {
                    message: "Turn canceled: queue was cleared by operator".to_string(),
                })
                .await;
        }
        info!("TurnQueue: cleared {} pending jobs", count);
        count
    }

    pub async fn get_status(&self) -> QueueStatusResponse {
        let max = self.max_concurrent.load(Ordering::Relaxed);
        let active = self.active.lock().await;
        let queue = self.queue.lock().await;

        let active_turns: Vec<ActiveTurnInfo> = active.values().cloned().collect();
        let queued_turns: Vec<QueuedTurnInfo> = queue
            .iter()
            .enumerate()
            .map(|(idx, j)| QueuedTurnInfo {
                turn_id: j.turn_id.clone(),
                session_id: j.session_id.clone(),
                mode: j.mode.clone(),
                source: j.source.clone(),
                queued_at: j.queued_at,
                position: idx + 1,
            })
            .collect();

        QueueStatusResponse {
            max_concurrent_turns: max,
            active_count: active_turns.len(),
            queued_count: queued_turns.len(),
            active_turns,
            queued_turns,
        }
    }

    async fn worker_loop(self: Arc<Self>) {
        loop {
            self.notify.notified().await;

            loop {
                let max = self.max_concurrent.load(Ordering::Relaxed);
                let mut active = self.active.lock().await;
                if active.len() >= max {
                    break;
                }

                let mut queue = self.queue.lock().await;
                let now = Utc::now();
                queue.retain(|j| {
                    if j.source.starts_with("tool_") && (now - j.queued_at).num_seconds() > 300 {
                        info!("Pruning stale queued background turn {} for session {} (age > 300s)", j.turn_id, j.session_id);
                        false
                    } else {
                        true
                    }
                });

                let job = match queue.pop_front() {
                    Some(j) => j,
                    None => break,
                };

                // Notify remaining waiting jobs of their updated queue positions
                for (idx, waiting) in queue.iter().enumerate() {
                    let _ = waiting.event_tx.send(AgentEvent::Queued {
                        turn_id: waiting.turn_id.clone(),
                        queue_position: idx + 1,
                        total_queued: queue.len(),
                    }).await;
                }
                drop(queue);

                let active_info = ActiveTurnInfo {
                    turn_id: job.turn_id.clone(),
                    session_id: job.session_id.clone(),
                    mode: job.mode.clone(),
                    source: job.source.clone(),
                    started_at: Utc::now(),
                };
                active.insert(job.turn_id.clone(), active_info);
                drop(active);

                // Notify session that queue wait has completed and inference has started
                let _ = job.event_tx.send(AgentEvent::QueueStarted {
                    turn_id: job.turn_id.clone(),
                }).await;

                let q_self = Arc::clone(&self);
                let agent = Arc::clone(&self.agent);

                tokio::spawn(async move {
                    let turn_id = job.turn_id;
                    let session_id = job.session_id;
                    let prompt = job.prompt;
                    let mode = job.mode;
                    let images = job.images;
                    let event_tx = job.event_tx;

                    info!("Starting execution of turn {} for session {}", turn_id, session_id);
                    if let Err(e) = agent.run_turn_with_images(&session_id, &prompt, &mode, images, event_tx).await {
                        error!("Error executing turn {} in session {}: {:?}", turn_id, session_id, e);
                    }
                    info!("Completed execution of turn {} for session {}", turn_id, session_id);

                    // Turn finished: remove from active and wake up supervisor for next job
                    let mut active = q_self.active.lock().await;
                    active.remove(&turn_id);
                    drop(active);

                    q_self.notify.notify_one();
                });
            }
        }
    }
}
