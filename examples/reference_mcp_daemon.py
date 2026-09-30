#!/usr/bin/env python3
"""
Reference Model Context Protocol (MCP) Daemon
Demonstrates a micro-daemon with both Autonomous (read-only) and Gated (mutating) tools
communicating via JSON-RPC 2.0 over stdio.
"""

import sys
import json
import os
import platform
import time

SERVER_INFO = {
    "name": "styx-reference-daemon",
    "version": "1.0.0"
}

TOOLS = [
    {
        "name": "system_health_check",
        "description": "Inspect host system health, memory stats, load average, and kernel platform (Autonomous / Read-only).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "detailed": {
                    "type": "boolean",
                    "description": "Whether to include detailed per-core breakdown"
                }
            }
        }
    },
    {
        "name": "send_alert_dispatch",
        "description": "Dispatch a high-priority incident alert to an external notification channel (Gated / Mutating: halts for human sign-off).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "channel": {
                    "type": "string",
                    "description": "Target dispatch channel (e.g. '#incident-ops', 'pagerduty', 'security-alerts')"
                },
                "severity": {
                    "type": "string",
                    "enum": ["P0_CRITICAL", "P1_HIGH", "P2_NORMAL"],
                    "description": "Alert severity level"
                },
                "summary": {
                    "type": "string",
                    "description": "Single-line summary of incident"
                },
                "message_body": {
                    "type": "string",
                    "description": "Full diagnostic message and remediation recommendations"
                }
            },
            "required": ["channel", "severity", "summary", "message_body"]
        }
    },
    {
        "name": "restart_service",
        "description": "Restart an operational daemon or service container (Gated / Mutating: halts for human sign-off).",
        "inputSchema": {
            "type": "object",
            "properties": {
                "service_name": {
                    "type": "string",
                    "description": "Name of service daemon (e.g. 'vllm-engine', 'docker', 'nginx')"
                },
                "graceful": {
                    "type": "boolean",
                    "description": "Send SIGTERM before SIGKILL"
                }
            },
            "required": ["service_name"]
        }
    }
]

def handle_request(req):
    method = req.get("method")
    req_id = req.get("id")
    params = req.get("params", {})

    if method == "initialize":
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {"listChanged": True}
                },
                "serverInfo": SERVER_INFO
            }
        }

    elif method == "notifications/initialized":
        return None

    elif method == "tools/list":
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "tools": TOOLS
            }
        }

    elif method == "tools/call":
        tool_name = params.get("name")
        args = params.get("arguments", {})

        if tool_name == "system_health_check":
            load1, load5, load15 = os.getloadavg() if hasattr(os, "getloadavg") else (0.0, 0.0, 0.0)
            result_data = {
                "status": "HEALTHY",
                "platform": platform.platform(),
                "python_version": platform.python_version(),
                "load_average": {"1m": load1, "5m": load5, "15m": load15},
                "timestamp": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                "detailed": args.get("detailed", False)
            }
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "content": [{"type": "text", "text": json.dumps(result_data, indent=2)}],
                    "isError": False
                }
            }

        elif tool_name == "send_alert_dispatch":
            channel = args.get("channel", "default")
            severity = args.get("severity", "P1_HIGH")
            summary = args.get("summary", "")
            body = args.get("message_body", "")

            # Simulated alert dispatch
            dispatch_result = {
                "dispatch_id": f"alt_{int(time.time())}",
                "status": "DISPATCHED",
                "target_channel": channel,
                "severity": severity,
                "summary": summary,
                "dispatched_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
                "receipt": "OK 200 - Alert acknowledged by upstream webhook"
            }
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "content": [{"type": "text", "text": json.dumps(dispatch_result, indent=2)}],
                    "isError": False
                }
            }

        elif tool_name == "restart_service":
            svc = args.get("service_name")
            graceful = args.get("graceful", True)
            restart_result = {
                "status": "RESTARTED",
                "service": svc,
                "graceful": graceful,
                "exit_code": 0,
                "restarted_at": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())
            }
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "content": [{"type": "text", "text": json.dumps(restart_result, indent=2)}],
                    "isError": False
                }
            }

        else:
            return {
                "jsonrpc": "2.0",
                "id": req_id,
                "error": {
                    "code": -32601,
                    "message": f"Method / tool '{tool_name}' not found"
                }
            }

    else:
        return {
            "jsonrpc": "2.0",
            "id": req_id,
            "error": {
                "code": -32601,
                "message": f"Unknown method: {method}"
            }
        }

def main():
    # Make sure stdout is unbuffered
    sys.stdout.reconfigure(line_buffering=True)
    sys.stderr.reconfigure(line_buffering=True)

    for line in sys.stdin:
        line = line.strip()
        if not line:
            continue
        try:
            req = json.loads(line)
            resp = handle_request(req)
            if resp is not None:
                sys.stdout.write(json.dumps(resp) + "\n")
                sys.stdout.flush()
        except Exception as e:
            err_resp = {
                "jsonrpc": "2.0",
                "id": None,
                "error": {"code": -32700, "message": f"Parse error: {str(e)}"}
            }
            sys.stdout.write(json.dumps(err_resp) + "\n")
            sys.stdout.flush()

if __name__ == "__main__":
    main()
