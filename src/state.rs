use std::sync::Arc;
use tokio::sync::broadcast;

use crate::agent::AgentRunner;
use crate::config::AppConfig;
use crate::db::Database;
use crate::docker::DockerOrchestrator;
use crate::hitl::HitlGate;
use crate::mcp::McpRegistry;
use crate::memory::MemoryManager;
use crate::router::MultiModelRouter;
use crate::telemetry::TelemetryCollector;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub db: Database,
    pub memory: MemoryManager,
    pub docker: DockerOrchestrator,
    pub mcp: McpRegistry,
    pub hitl: HitlGate,
    pub router: MultiModelRouter,
    pub telemetry: TelemetryCollector,
    #[allow(dead_code)]
    pub agent: Arc<AgentRunner>,
    pub turn_queue: Arc<crate::agent::queue::TurnQueue>,
    pub ws_broadcast: broadcast::Sender<String>,
}

impl AppState {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        config: AppConfig,
        db: Database,
        memory: MemoryManager,
        docker: DockerOrchestrator,
        mcp: McpRegistry,
        hitl: HitlGate,
        router: MultiModelRouter,
        telemetry: TelemetryCollector,
    ) -> Self {
        let agent = Arc::new(AgentRunner::new(
            db.clone(),
            memory.clone(),
            mcp.clone(),
            hitl.clone(),
            router.clone(),
            telemetry.clone(),
            docker.clone(),
        ));

        let turn_queue = crate::agent::queue::TurnQueue::new(agent.clone(), config.max_concurrent_turns);

        let (ws_broadcast, _) = broadcast::channel(1024);

        Self {
            config,
            db,
            memory,
            docker,
            mcp,
            hitl,
            router,
            telemetry,
            agent,
            turn_queue,
            ws_broadcast,
        }
    }
}
