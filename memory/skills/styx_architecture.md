# Skillset: Styx Architecture, Container Execution & Memory Modification

## Objective
Provide the agent with operational mastery over the Styx Personal AI OS architecture, container sandbox execution, memory management, and reactive tool coordination.

## Core Architecture Pillars

### 1. Dedicated Single-User Personal Device Model
- Styx is architected like a personal smartphone or workstation: one user per hosting instance.
- It operates with full loyalty and dedication to a single operator.
- Access is secured at the perimeter via the Device Access Key.

### 2. Container Sandbox vs Host Boundary
- **Inside Container (`/app`):**
  - Styx Agent Harness, SQLite database, Tantivy BM25 search engine, and local model runner.
  - The agent has autonomous execution privileges via `exec_container_command`.
  - Can write temporary scripts in `/app/scratchpad` or `/tmp` to compute mathematical answers, test python logic, parse JSON/logs, or inspect runtime environments.
- **Outside Container (Host System):**
  - Host tools (desktop automation, messaging daemons, local file scanners, external services).
  - External tools send reactive events to `POST /api/tools/trigger` or listen via MCP stdio/Unix sockets.
  - External mutating side effects deterministically trigger the Human-in-the-Loop (HITL) approval gate.

### 3. Memory & System Context Ingestion
- `core/system_instructions.md` and `core/user_profile.md` are concatenated and sent with **every turn prompt**.
- The agent does not need to waste tool calls looking up basic operator facts.
- **Memory Modification Protocol:**
  - When the operator reveals new workflow habits, project updates, or communication preferences:
    - Call `write_memory(path="core/user_profile.md", content=...)` or `write_memory(path="core/system_instructions.md", content=...)`.
    - This updates the markdown file and triggers a Tantivy BM25 re-indexing in real time.
    - Future turns automatically reflect the updated context.

### 4. GPU Turn Queue & Concurrency Management
- Hardware inference slots are controlled by `TurnQueue`.
- On single-GPU systems (`max_concurrent_turns = 1`), turns are processed sequentially in strict FIFO order, preventing GPU VRAM exhaustion or thrashing.
- Turns are tracked with real-time queue position indicators.

### 5. Onboarding Task Protocol
- When `core/user_profile.md` indicates unconfigured operator status:
  - Greet the user warmly and concisely.
  - Ask for name/handle, role, preferred communication style, timezone, and immediate goals.
  - Immediately save the captured profile to `core/user_profile.md`.
