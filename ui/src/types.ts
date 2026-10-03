export interface GpuTelemetry {
  name: string;
  vendor?: 'amd' | 'nvidia' | 'none' | 'auto';
  vram_used_mb: number;
  vram_total_mb: number;
  vram_pct: number;
  gpu_util_pct: number;
  temperature_c?: number;
}

export interface SystemTelemetry {
  host_cpu_pct: number;
  host_cpu_cores: number;
  memory_used_mb: number;
  memory_total_mb: number;
  memory_pct: number;
  disk_used_gb: number;
  disk_total_gb: number;
  disk_pct: number;
  uptime_secs: number;
  running_containers: number;
  active_mcp_servers: number;
  pending_approvals: number;
  active_model: string;
  tokens_per_second: number;
  gpu?: GpuTelemetry | null;
  queued_turns?: number;
  active_turns?: number;
  max_concurrent_turns?: number;
}

export interface QueuedTurnInfo {
  turn_id: string;
  session_id: string;
  mode: string;
  source: string;
  queued_at: string;
  position: number;
}

export interface ActiveTurnInfo {
  turn_id: string;
  session_id: string;
  mode: string;
  source: string;
  started_at: string;
}

export interface QueueStatusResponse {
  max_concurrent_turns: number;
  active_count: number;
  queued_count: number;
  active_turns: ActiveTurnInfo[];
  queued_turns: QueuedTurnInfo[];
}

export interface ApprovalTicket {
  ticket_id: string;
  session_id: string;
  tool_name: string;
  arguments: any;
  risk_level: string; // "LOW" | "HIGH" | "CRITICAL"
  explanation?: string;
  created_at: string;
}

export interface ChatSession {
  id: string;
  title: string;
  mode: string; // "chat" | "mission"
  created_at: string;
  updated_at: string;
}

export interface ChatMessage {
  id: string;
  session_id: string;
  role: 'user' | 'assistant' | 'system' | 'tool';
  content: string;
  thought?: string | null;
  tool_calls?: string | null;
  created_at: string;
}

export interface MemoryFileNode {
  path: string;
  category: string;
  filename: string;
  title: string;
  size_bytes: number;
  updated_at: string;
}

export interface ContainerSummaryInfo {
  id: string;
  names: string[];
  image: string;
  status: string;
  state: string;
  ports: string[];
  is_syndae_managed: boolean;
  created: number;
  model_id?: string | null;
}

export interface EnginePreset {
  id: string;
  label: string;
  engine: 'vllm' | 'llamacpp' | 'ollama';
  hf_repo: string;
  model_size_b?: number;
  default_context: number;
  default_port: number;
  recommended_gpu: string;
  default_quantization?: string;
  default_tp: number;
  enable_mtp?: boolean;
  enable_vision?: boolean;
  description: string;
}

export interface QuantizationRecommendation {
  target_gpu_vram_gb: number;
  recommended_quantization: string;
  fits_comfortably: boolean;
  estimated_weights_gb: number;
  estimated_kv_cache_gb: number;
  estimated_total_gb: number;
  rationale: string;
}

export interface McpTool {
  name: string;
  description: string;
  input_schema: any;
  server_id?: string;
  policy: 'AUTONOMOUS' | 'REQUIRE_APPROVAL' | 'BLOCKED';
  risk_level: string;
}

export interface McpServerRecord {
  id: string;
  name: string;
  transport_type: string;
  command?: string;
  args_json?: string;
  env_json?: string;
  socket_path?: string;
  url?: string;
  enabled: boolean;
  created_at: string;
}

export interface AuditEventRecord {
  id: string;
  session_id?: string;
  event_type: string;
  tool_name?: string;
  payload_json?: string;
  decision?: string;
  duration_ms?: number;
  created_at: string;
}

export interface ModelConfigRecord {
  id: string;
  name: string;
  provider: string;
  base_url?: string;
  api_key?: string;
  model_id: string;
  context_length: number;
  is_active: boolean;
  extra_flags_json?: string;
  created_at: string;
}

export interface AuthStatus {
  initialized: boolean;
  onboarded: boolean;
  operator_name: string | null;
  requires_auth: boolean;
}

export interface InspectedToolInfo {
  name: string;
  display_name: string;
  version: string;
  description: string;
  path: string;
  has_docker: boolean;
  has_compose: boolean;
  container_name: string;
  container_exists: boolean;
  container_running: boolean;
  container_status: string;
  container_healthy: boolean;
  port: number;
  recommended_transport: string;
  recommended_url: string;
  is_registered: boolean;
  tools: {
    name: string;
    description: string;
    risk_level: string;
    policy: 'AUTONOMOUS' | 'REQUIRE_APPROVAL' | 'BLOCKED';
  }[];
  skill_file?: string;
  has_instructions?: boolean;
  instructions?: string;
}

export interface DiscoveredLocalTool {
  id: string;
  name: string;
  display_name: string;
  version: string;
  description: string;
  path: string;
  is_installed: boolean;
  has_docker: boolean;
}

export type ChannelTriggerPolicy = 'all' | 'mentions' | 'muted' | 'manual';

export interface ChannelPolicyRecord {
  channel_id: string;
  protocol: string;
  channel_name: string | null;
  is_group: boolean;
  policy: ChannelTriggerPolicy;
  mention_keywords: string | null;
  updated_at: string;
}

export interface ChannelDefaultsRecord {
  default_group_policy: ChannelTriggerPolicy;
  default_direct_policy: ChannelTriggerPolicy;
  mention_keywords: string;
}



