# Skillset: WhatsApp Connector & Messaging Integration

## 1. Tool Overview & Architecture
The WhatsApp Connector is an isolated micro-daemon running alongside Styx Agent OS. It bridges Baileys WhatsApp Web sockets to standard Model Context Protocol (MCP 2024-11-05) and relays reactive inbound events to Styx via HTTP webhooks.

### Integration Channels:
- **Inbound Events**: Dispatched to `POST /api/tools/trigger` with `protocol: "whatsapp"`. Styx automatically routes incoming messages from the same sender into a persistent, multi-turn chat session.
- **Outbound Execution**: Dispatched by Styx agent calling MCP tools over JSON-RPC at `http://host.docker.internal:8765/mcp`.
- **Human-in-the-Loop (HITL) Gate**: Mutating outbound actions (`send_message`, `send_sticker`, `send_reaction`) require explicit operator confirmation before execution.

---

## 2. Tool Catalog & Best Practices

### `find_stickers` (Autonomous, Risk: LOW)
- **Parameters**: `search: string`, `tags?: string[]`, `limit?: number`
- **Purpose**: Search the local sticker catalog by emotion, intent, reaction, or keyword (e.g. "shrug", "thumbs up", "laugh", "party", "fire", "celebrate").
- **Best Practice**: ALWAYS run `find_stickers` first before proposing a sticker reaction to discover valid `sticker_id`s.

### `send_sticker` (Requires Approval, Risk: HIGH)
- **Parameters**: `to: string`, `sticker_id: string`
- **Purpose**: Sends a rich WebP sticker or animated sticker/GIF to a contact or group.
- **`sticker_id`**: Accepts valid catalog IDs from `find_stickers` (e.g. `cat_shrug`, `fire_lit`) OR a direct image/GIF/WebP URL (such as returned by `download_sticker_as_webp` or web search). NEVER invent non-existent IDs like `green_lizard_wave`; if sending a web sticker or GIF, pass the URL returned by `download_sticker_as_webp` directly.
- **Recipient (`to`)**: Accepts phone numbers (`+1234567890`), JIDs (`...@s.whatsapp.net` or `...@lid`), group JIDs (`...@g.us`), or contact names.

### `send_message` (Requires Approval, Risk: HIGH)
- **Parameters**: `to: string`, `message: string`
- **Purpose**: Sends a plain text message to an individual or group.
- **Best Practice**:
  - Keep suggested replies concise (1-3 sentences).
  - Match the recipient's language, dialect, and communication style as documented in `people/<name>.md`.
  - Never include robotic AI disclaimers or unsolicited corporate filler.

### `send_reaction` (Requires Approval, Risk: HIGH)
- **Parameters**: `to: string`, `message_id: string`, `emoji: string`
- **Purpose**: Adds an emoji reaction (e.g. "👍", "❤️", "😂", "🔥") to a specific message.

### `get_chat_history` (Autonomous, Risk: LOW)
- **Parameters**: `chat?: string`, `limit?: number`
- **Purpose**: Retrieves recent inbound and outbound messages to maintain conversational context across turns.

### `list_chats` (Autonomous, Risk: LOW)
- **Parameters**: `limit?: number`
- **Purpose**: Lists active conversations, unread messages, and recent contact identifiers.

### `fetch_older_messages` (Autonomous, Risk: LOW)
- **Parameters**: `chat: string`, `count?: number`
- **Purpose**: Requests historical message synchronization from the primary mobile phone via on-demand peer sync.

### `import_chat_export` (Autonomous, Risk: LOW)
- **Parameters**: `content: string`, `chat_name: string`
- **Purpose**: Ingests exported `.txt` chat logs into the local database with chronologic deduplication.

### `get_whatsapp_status` (Requires Approval, Risk: CRITICAL)
- **Parameters**: none
- **Purpose**: Inspects bridge state (`connected`, `connecting`, `qr_ready`, `disconnected`), battery, and paired account info.

---

## 3. Protocol for Reactive Inbound Messages
When an inbound message event arrives:
1. **Never Speak Directly to External Senders**: External contacts do not see your Styx chat output. Always address your operator.
2. **Context & Tone Retrieval**:
   - Check `people/<sender>.md` and `groups/<group>.md` for relationship dynamics, history, and preferred language.
   - Check `dictionary/*.md` for shared slang or shorthand.
3. **Drafting Suggested Action**:
   - Propose an authentic, natural reply or reaction.
   - Conclude with 3-5 distinct choices formatted in `<options>` tags (e.g. send suggested reply, send sticker, adjust tone, dismiss).
4. **Autonomous World Learning**:
   - If the contact or group is new, or if new personal details / vocabulary are observed, use `write_memory` to update `people/<name>.md`, `groups/<group>.md`, or `dictionary/*.md`.
