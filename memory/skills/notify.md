# Local Device Notifications MCP Tool

## Overview
The `notify` tool enables Styx agents to dispatch 100% local notifications directly to the operator's devices without relying on Google FCM, Apple APNs, or third-party cloud services.

## Operational Modes
1. **Local Self-Hosted Push (ntfy / Gotify)**:
   - If `NTFY_URL` or `GOTIFY_URL` is configured (e.g., `http://192.168.1.2:8080/styx-alerts`), the tool dispatches directly to your local push server over your home LAN.
   - Phones running the open-source F-Droid ntfy/Gotify app receive notifications instantly, vibrating and ringing even if the browser is closed.
2. **Local Desktop Alert (`notify-send` / System Alert)**:
   - On Linux/macOS/Windows desktop hosts, triggers native OS desktop notifications directly without network overhead.
3. **Styx Inbound Relay**:
   - Forwards inbound notifications to Styx generic gateway (`POST /api/tools/trigger`) to ensure session continuity.

## Tool Definition
- `send_notification`:
  - `title` (string, required): Short summary of the alert.
  - `message` (string, required): Full notification body explaining what is needed or what completed.
  - `urgency` (string, optional: "info" | "action_required" | "alert"): Urgency tier. Defaults to "info".
  - `endpoint` (string, optional): Specific local HTTP webhook destination. Defaults to environment config.
