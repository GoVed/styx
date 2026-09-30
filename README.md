# STYX // Agent Harness, Model Manager & Mission Control Dashboard

**Styx** is a zero-overhead personal AI operating system and runtime written in Rust, designed to run natively inside a Linux VM or container host.

```
========================================================================
   ███████╗████████╗██╗   ██╗██╗  ██╗
   ██╔════╝╚══██╔══╝╚██╗ ██╔╝╚██╗██╔╝
   ███████╗   ██║    ╚████╔╝  ╚███╔╝ 
   ╚════██║   ██║     ╚██╔╝   ██╔██╗ 
   ███████║   ██║      ██║   ██╔╝ ██╗
   ╚══════╝   ╚═╝      ╚═╝   ╚═╝  ╚═╝
   AGENT HARNESS, MODEL MANAGER & MISSION CONTROL DASHBOARD
========================================================================
```

---

## Architectural Overview

Styx unifies personal agent orchestration, local Dockerized inference engines, dynamic Model Context Protocol (MCP) micro-daemons, and human-in-the-loop governance into a single, cohesive binary.

```
┌────────────────────────────────────────────────────────────────────────┐
│                   STYX MISSION CONTROL DASHBOARD                       │
│  [Telemetry Strip] [Chat & Missions] [Memory Hub] [Models] [Tool Bus]   │
└────────────────────────────────────▲───────────────────────────────────┘
                                     │ (HTTP REST + Bi-directional WebSockets)
┌────────────────────────────────────▼───────────────────────────────────┐
│                           STYX BACKEND (RUST)                          │
│                                                                        │
│  ┌───────────────────────┐             ┌─────────────────────────────┐ │
│  │   Multi-Model Router  │             │   Deterministic HITL Gate   │ │
│  │  - Local vLLM/Ollama  │             │  - tokio::sync::oneshot     │ │
│  │  - Anthropic / Gemini │             │  - Zero-timeout suspension  │ │
│  │  - OpenAI Compatible  │             │  - Approve / Reject / Edit  │ │
│  └───────────▲───────────┘             └──────────────▲──────────────┘ │
│              │                                        │                │
│  ┌───────────▼───────────┐             ┌──────────────▼──────────────┐ │
│  │     Agent Runtime     │◄───────────►│       MCP Tool Bus          │ │
│  │  - Chat Prompt Mode   │             │  - stdio child processes    │ │
│  │  - Autonomous Mission │             │  - Unix domain sockets      │ │
│  │  - Reasoning Stream   │             │  - 3-tier policy matrix     │ │
│  └───────────▲───────────┘             └─────────────────────────────┘ │
│              │                                                         │
│  ┌───────────▼───────────┐             ┌─────────────────────────────┐ │
│  │   Memory Engine       │             │   Docker Provisioner        │ │
│  │  - /memory/ directory │             │  - bollard (Docker Engine)  │ │
│  │  - Tantivy BM25 FTS   │             │  - vLLM / llama.cpp / Ollama│ │
│  │  - Transparent & Git  │             │  - MTP / Speculative flags  │ │
│  └───────────────────────┘             └─────────────────────────────┘ │
└────────────────────────────────────────────────────────────────────────┘
```

---

## Core Subsystems

### 1. Live Conversational Chat & Autonomous Mission View
- **Dual-Mode Central Workspace:**
  - **Conversational Prompt Mode:** Interactive session supporting token streaming, thought / reasoning blocks (`<think>...</think>`), and tool status callouts.
  - **Autonomous Mission Mode:** Multi-step autonomous agent loop that formulates plans, consults memory files, issues tool executions, checks results, and updates scratchpads until the objective is reached.
- **Inline Action Blocks:** Expandable execution cards rendered directly in the stream showing invocation arguments, execution duration, and output payload.
- **Inline Approval Cards:** When an action is mutating, execution halts and renders an interactive confirmation banner within the conversation stream (`[Approve & Continue]`, `[Reject]`, `[Edit Arguments]`).

### 2. Markdown-Based Knowledge & Memory Store (`/memory/`)
A completely transparent, file-based memory system editable with standard text editors and version-controlled via git:
- `/memory/core/`: Long-term user preferences, profile context, and operational guidelines (`user_profile.md`, `system_instructions.md`).
- `/memory/skills/`: Modular skillsets and heuristics dynamically installed by connected tool packages or created by the operator.
- `/memory/scratchpad/`: Daily working notes and active project scratchpads (`daily_log.md`, `active_projects.md`).
- **Memory Engine Tools:**
  - `read_memory(path)`: Autonomous read-only access.
  - `search_memory(query, limit)`: Fast embedded full-text search over markdown files powered by **Tantivy (BM25)**.
  - `write_memory(path, content, section)`: State-mutating action requiring human operator sign-off.
- **In-App Memory Hub:** Integrated tree browser and markdown editor with live rendered preview and Tantivy test search bar.

### 3. Containerized Model Lifecycle Manager
Directly deploy and configure local LLM inference engines inside Docker from the UI using Hugging Face repositories:
- **Engines Supported:**
  - **vLLM** (`vllm/vllm-openai:latest`): High-throughput OpenAI-compatible server.
  - **llama.cpp** (`ghcr.io/ggerganov/llama.cpp:server`): Lightweight CPU/GPU GGUF server.
  - **Ollama** (`ollama/ollama:latest`): Flexible containerized Ollama server.
- **Advanced Configuration Flags:**
  - Context window size (`--max-model-len` / `-c`).
  - Speculative decoding & Multi-Token Prediction (MTP) flags (`--speculative-model`, `--num-speculative-tokens`).
  - Hardware & Performance flags: GPU Device selection (`--gpus all` or device IDs), Tensor Parallel Size (`-tp`), GPU Memory Utilization (`--gpu-memory-utilization`), Quantization (`awq`, `gptq`, `fp8`, `k-quants`).
  - Secure Hugging Face token injection for gated weights (`HUGGING_FACE_HUB_TOKEN`).
- **Docker Run Preview:** Interactive CLI command generator displaying the exact generated command for full transparency.
- **Lifecycle & Logs:** Start, stop, restart, delete, inspect containers, and stream real-time container logs (for weight downloads and CUDA initialization).

### 4. Flexible Multi-Model Router
- Seamlessly routes between:
  - Local containerized engines (vLLM, llama.cpp, Ollama).
  - External OpenAI-compatible endpoints (`/v1/chat/completions`).
  - Cloud Providers: Anthropic Claude (Messages API with streaming & tool use), Google Gemini, and OpenAI.
- Real-time connection handshake test button validating network latency and supported context lengths.

### 5. Pluggable Tool Bus (Model Context Protocol - MCP)
- No hardcoded external tools in the harness. Connects external micro-daemons dynamically via:
  - `stdio`: Child process execution over stdin/stdout JSON-RPC 2.0.
  - `unix_socket`: Unix domain sockets (`/var/run/*.sock`).
  - `http_sse`: HTTP/SSE endpoints.
- Auto-discovers schemas via `tools/list` on registration and incorporates them into the LLM context.
- **Three-Tier Policy Matrix:**
  - `AUTONOMOUS`: Read-only queries (`read_memory`, `search_memory`, `get_system_telemetry`, `system_health_check`) execute immediately.
  - `REQUIRE_APPROVAL`: State-mutating actions (`write_memory`, `send_alert_dispatch`, `restart_service`, `send_email`, `exec_bash`) halt execution for human confirmation.
  - `BLOCKED`: Excluded from the model context.

### 6. Deterministic Human-in-the-Loop (HITL) Gate
- Built outside the LLM context in deterministic Rust. Prompt injection cannot bypass this gate.
- Mutating actions trigger an `ApprovalTicket` broadcast via WebSocket.
- Execution task suspends non-blockingly using `tokio::sync::oneshot` channels without timing out.
- The human can:
  - **Approve:** Continue execution with original arguments.
  - **Reject:** Abort tool execution and return user feedback to the LLM.
  - **Modify Payload:** Interactively edit parameters in the UI before execution.

---

## Quickstart Guide

### Prerequisites
- Linux VM / Host (Fedora, Debian, Ubuntu, Arch)
- Rust toolchain (`cargo`, `rustc` 1.80+)
- Node.js (v20+) & npm
- Docker Engine & `/var/run/docker.sock` (optional, for local container provisioner)
- Python 3 (for reference MCP daemon)

---

### Native VM Deployment

1. **Clone the repository:**
   ```bash
   git clone https://github.com/your-org/styx.git
   cd styx
   ```

2. **Build the Frontend UI:**
   ```bash
   cd ui
   npm install
   npm run build
   cd ..
   ```

3. **Run the Styx Runtime:**
   ```bash
   cargo run
   ```

4. **Access Mission Control:**
   Open your browser to:
   ```
   http://localhost:3000
   ```

---

### Docker & Docker Compose Deployment

To deploy Styx inside Docker with host Docker socket passthrough:

```bash
docker compose up -d --build
```

Then navigate to `http://localhost:3000`.

---

## Reference Tool Daemon (`examples/reference_mcp_daemon.py`)

Styx includes a standalone Python reference daemon demonstrating the Model Context Protocol:

```bash
# Test the reference daemon directly over stdio
python3 examples/reference_mcp_daemon.py
```

The reference daemon exposes:
1. `system_health_check` (*Autonomous / Read-only*): Inspects host load average, platform details, and memory.
2. `send_alert_dispatch` (*Gated / Mutating*): Dispatches operational incident alerts. Halts for human sign-off.
3. `restart_service` (*Gated / Mutating*): Issues a restart command for a host system service. Halts for human sign-off.

---

## REST & WebSocket API Reference

| Endpoint | Method | Description |
| :--- | :--- | :--- |
| `/api/telemetry` | `GET` | Host VM CPU, RAM, Disk, and Docker stats |
| `/api/chat/sessions` | `GET`, `POST` | List and create chat/mission sessions |
| `/api/chat/sessions/:id/messages` | `GET` | Message history for a session |
| `/api/chat/send` | `POST` | Dispatch prompt turn (streams to WebSocket) |
| `/api/memory/tree` | `GET` | List all files in `/memory/` |
| `/api/memory/file` | `GET`, `POST`, `DELETE` | Read, update, and delete memory markdown files |
| `/api/memory/create` | `POST` | Create a new skill or scratchpad file |
| `/api/memory/search` | `GET` | Tantivy BM25 full-text search across memory |
| `/api/models/containers` | `GET` | List Docker containers on host |
| `/api/models/deploy` | `POST` | Deploy vLLM / llama.cpp / Ollama container |
| `/api/models/containers/:id/start` | `POST` | Start stopped container |
| `/api/models/containers/:id/stop` | `POST` | Stop running container |
| `/api/models/containers/:id/restart` | `POST` | Restart container |
| `/api/models/containers/:id/logs` | `GET` | Container stdout/stderr build/download logs |
| `/api/models/presets` | `GET` | List model deployment presets |
| `/api/models/preview-command` | `POST` | Generate Docker run CLI command preview |
| `/api/models/configs` | `GET`, `POST` | List and register external model providers |
| `/api/models/configs/:id/activate`| `POST` | Set active model provider |
| `/api/models/test-connection` | `POST` | Test model endpoint handshake & latency |
| `/api/tools/discover` | `GET` | Auto-detect local MCP tool packages |
| `/api/tools/inspect` | `POST` | Inspect tool package manifest and container status |
| `/api/tools/install` | `POST` | Deploy and connect local MCP daemon |
| `/api/tools/trigger` | `POST` | Inbound tool event webhook relay (WhatsApp, etc.) |
| `/api/tools/servers` | `GET`, `POST` | List and register MCP servers |
| `/api/tools/list` | `GET` | List all tools with policy matrix |
| `/api/tools/policy` | `POST` | Update tool policy (`AUTONOMOUS`, `REQUIRE_APPROVAL`, `BLOCKED`) |
| `/api/tools/call` | `POST` | Invoke tool directly for testing |
| `/api/hitl/tickets` | `GET` | List pending and historic approval tickets |
| `/api/hitl/decide` | `POST` | Submit decision (`APPROVE`, `REJECT`, `MODIFY`) |
| `/api/audit/events` | `GET` | Audit trail of tool invocations & sign-offs |
| `/ws` | `WS` | Central WebSocket stream (telemetry, chat, HITL, logs) |

---

## Directory Structure

```
styx/
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
├── docker-compose.yml
├── .env.example
├── ARCHITECTURE_RULES.md
├── README.md
├── memory/
│   ├── core/
│   │   ├── system_instructions.md
│   │   └── user_profile.md
│   ├── skills/
│   │   └── .gitkeep
│   └── scratchpad/
│       ├── active_projects.md
│       └── daily_log.md
├── examples/
│   └── reference_mcp_daemon.py
├── src/
│   ├── main.rs
│   ├── config.rs
│   ├── state.rs
│   ├── agent/
│   │   ├── compression.rs
│   │   ├── diagnosis.rs
│   │   ├── execution.rs
│   │   ├── queue.rs
│   │   ├── reasoning.rs
│   │   ├── recovery.rs
│   │   ├── tools.rs
│   │   └── types.rs
│   ├── api/
│   │   ├── auth/
│   │   ├── models/
│   │   ├── tools/
│   │   ├── audit.rs
│   │   ├── chat.rs
│   │   ├── hitl.rs
│   │   ├── memory.rs
│   │   ├── telemetry.rs
│   │   └── ws.rs
│   ├── db/
│   │   ├── auth.rs
│   │   ├── messages.rs
│   │   ├── models.rs
│   │   ├── sessions.rs
│   │   └── tools.rs
│   ├── docker/
│   │   ├── deploy.rs
│   │   └── engines/
│   ├── hitl/
│   │   └── mod.rs
│   ├── mcp/
│   │   ├── builtin.rs
│   │   ├── policy.rs
│   │   ├── transport.rs
│   │   └── types.rs
│   ├── memory/
│   │   ├── merge.rs
│   │   └── search.rs
│   ├── router/
│   │   ├── anthropic.rs
│   │   ├── gemini.rs
│   │   ├── openai.rs
│   │   ├── streaming.rs
│   │   ├── types.rs
│   │   └── vision.rs
│   └── telemetry/
│       └── mod.rs
└── ui/
    ├── package.json
    ├── vite.config.ts
    ├── index.html
    └── src/
        ├── App.tsx
        ├── types.ts
        ├── index.css
        └── components/
            ├── AccessGate.tsx
            ├── ChatView.tsx
            ├── MemoryHub.tsx
            ├── ModelManager.tsx
            ├── SettingsHub.tsx
            ├── TelemetryStrip.tsx
            ├── ToolHub.tsx
            ├── access-gate/
            ├── chat/
            ├── memory/
            ├── model-manager/
            ├── navigation/
            ├── settings/
            └── tool-hub/
```

---

## License
MIT License. Built for zero-overhead personal AI operations on Linux.
