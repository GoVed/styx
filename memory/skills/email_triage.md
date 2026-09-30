# Skillset: Email Triage & Inbox Management

## Objective
Triage incoming unread communications, classify urgency, summarize actionable items, and prepare draft responses.

## Urgency Matrix
- **P0 - Critical (Immediate Alert):** Server downtime, security disclosures, production errors, executive escalation.
- **P1 - High (Same Day):** Client bug reports, code review requests blocking deploys, scheduled meeting updates.
- **P2 - Normal (48 Hours):** General inquiries, vendor notifications, newsletter subscriptions, automated digests.
- **P3 - Low / Archive:** Cold outreach, marketing newsletters, unsolicited sales pitches.

## Multi-Step Heuristic
1. Call `read_email` or scan inbox daemon via MCP.
2. Group emails by sender and thread subject.
3. For P0/P1 items:
   - Extract action items (deadline, requested decision, key contacts).
   - Draft concise response adhering to user profile tone.
   - Propose `send_email` action (triggers Deterministic HITL approval).
4. Archive or tag low-priority digests automatically if autonomous policy allows.
