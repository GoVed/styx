# Skillset: Messaging Tone, Reactive Etiquette & Reaction Protocol

## Objective
Govern personal and semi-professional communication etiquette for connected messaging tools and chat bridges when responding to reactive inbound tool events.

## Protocol Workflow
When an inbound message event arrives:
1. **Analyze Sender & Intent:**
   - Extract sender identifier and message intent (invitation, question, urgent task, casual banter).
2. **Context & Relationship Lookup:**
   - Check `people/<sender>.md` and `groups/<group>.md` for relationship, known communication style, language, and shared history.
   - Check `scratchpad/daily_log.md` and `core/user_profile.md` for current schedule, commitments, and operator availability.
3. **Conversational Style & Etiquette:**
   - **Brevity:** Keep suggested replies concise (1-3 sentences or short lines). Avoid corporate filler unless writing to a formal client.
   - **Match Tone & Language:** Match the sender's language, dialect, and energy (casual slang, formal courteous, witty banter, or emojis) based on `people/<sender>.md`.
4. **Sticker & Reaction Protocol:**
   - If context calls for an emotional reaction (e.g. declining an outing with friends, celebrating success, or acknowledging a funny meme):
     - First call `find_stickers(search="...")` with emotional context (e.g. "thumbs up", "celebrate", "laugh", "shrug").
     - Inspect the returned sticker IDs.
     - Choose a sticker matching the operator's personality.
     - Dispatch `send_sticker` or propose it to the operator.
5. **Continuous Learning:**
   - If this is a new contact or group, learn the relationship and communication dynamics and save notes to `people/<sender>.md` or `groups/<group>.md`.
