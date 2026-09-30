# STYX ARCHITECTURAL RULES & CORE LAWS

## Law 1: Styx Core is a Pure Harness
The Styx repository (`Styx/`) is strictly a general-purpose AI Operating System and execution harness.
- It contains ONLY:
  - Agent runtime, turn queue, reasoning / thinking loop.
  - Model management and local engine orchestration (vLLM, llama.cpp, Ollama).
  - Memory indexing (Tantivy BM25) and markdown file operations.
  - State database (SQLite) and configuration.
  - MCP client protocol (JSON-RPC) and generic HTTP inbound event gateway.
  - User interface (Chat, ModelManager, MemoryHub, ToolHub, Settings).
- It MUST NOT CONTAIN:
  - Any tool-specific service implementations, custom communication protocols, or third-party client drivers.
  - Any user-specific hardcoded names, personal identifiers, or profile assumptions.
  - Any language-specific, dialect-specific, or locale-specific hardcoded modules.
  - Any group-specific, topic-specific, or community-specific hardcoded logic.

## Law 2: Tools Live Exclusively in External Packages
- Tools are independent micro-daemons residing in external tool packages or standalone repositories.
- Tools communicate with Styx strictly via:
  1. Standard MCP protocol (`tools/list`, `tools/call`).
  2. Generic HTTP inbound webhook (`POST /api/tools/trigger`).
- Styx core remains 100% agnostic to what any external tool does. Styx only processes standardized parameters: `protocol`, `channel_id`, `source_id`, `event_type`, `payload`.

## Law 3: Universal World-Learning Framework
- Every user has a completely distinct life, culture, language, family, and social circle.
- Styx is built for millions of users worldwide and makes zero assumptions about who the user is.
- Styx dynamically learns about each user's world and records it into `/memory/`:
  - `core/user_profile.md`: The user's identity, routines, languages, and general preferences.
  - `people/<contact_name>.md`: The user's relationship with that specific person, communication style, language, and shared history.
  - `groups/<group_name>.md`: The purpose behind the group, its dynamics, tone, and participants.
  - `dictionary/<topic>.md`: Slang, colloquial expressions, and vernacular shorthand used in their circles.
- The agent has full authority to save, enrich, and maintain these notes autonomously (`write_memory`).

## Law 4: Code Cleanliness & Constraints
- Maximum ~300-350 LOC per file. If a file grows, split cleanly into submodules.
- 100% test coverage preserved across unit, UI, and integration test suites.
