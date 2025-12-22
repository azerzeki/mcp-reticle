#!/usr/bin/env python3
"""
Test MCP server that demonstrates error handling.
This script outputs both valid JSON-RPC and error messages to test Reticle.
"""

import sys
import json
import time

def send_jsonrpc(obj):
    """Send a JSON-RPC message to stdout."""
    print(json.dumps(obj), flush=True)

def send_stderr(msg):
    """Send an error message to stderr."""
    print(msg, file=sys.stderr, flush=True)

def main():
    # Send some stderr output first (like Python warnings on startup)
    send_stderr("Warning: This is a test warning from the MCP server")
    send_stderr("DEBUG: Server starting up...")

    # Send a valid JSON-RPC notification
    send_jsonrpc({
        "jsonrpc": "2.0",
        "method": "notifications/initialized"
    })

    time.sleep(0.5)

    # Send raw stdout (non-JSON)
    print("This is raw stdout output - not JSON-RPC!", flush=True)
    print("Another raw line with some debug info", flush=True)

    time.sleep(0.5)

    # Simulate a Python exception/traceback
    send_stderr("""Traceback (most recent call last):
  File "/path/to/mcp_server.py", line 42, in handle_request
    result = process_tool_call(params)
  File "/path/to/mcp_server.py", line 78, in process_tool_call
    raise ValueError("Invalid tool arguments")
ValueError: Invalid tool arguments""")

    time.sleep(0.5)

    # Send another valid JSON-RPC message
    send_jsonrpc({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "tools": [
                {"name": "test_tool", "description": "A test tool"}
            ]
        }
    })

    time.sleep(0.5)

    # More stderr
    send_stderr("ERROR: Something went wrong!")
    send_stderr("CRITICAL: Database connection failed")

    # Final JSON-RPC
    send_jsonrpc({
        "jsonrpc": "2.0",
        "id": 2,
        "error": {
            "code": -32603,
            "message": "Internal error",
            "data": {"details": "Connection timeout"}
        }
    })

    # Keep running briefly so the proxy can read all output
    time.sleep(1)

if __name__ == "__main__":
    main()
