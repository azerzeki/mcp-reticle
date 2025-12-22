#!/usr/bin/env python3
"""
Mock MCP Server with SSE (Server-Sent Events) Transport

This script implements a mock MCP server that uses HTTP/SSE instead of stdio.
It's useful for testing Reticle's SSE proxy capabilities.

Usage:
    python3 mock-mcp-sse-server.py --port 8080
    curl -N http://localhost:8080/events

Features:
    - SSE endpoint at /events
    - Realistic MCP protocol messages
    - Configurable message frequency
    - Heartbeat to keep connection alive
"""

from flask import Flask, Response, request, jsonify
from flask_cors import CORS
import json
import time
import argparse
import random
import sys

app = Flask(__name__)
CORS(app)  # Enable CORS for browser clients

# Configuration
CONFIG = {
    'iterations': 10,
    'delay': 1.0,
    'mode': 'realistic',
}

def format_sse_event(data, event_type='message', event_id=None):
    """
    Format data as Server-Sent Event

    SSE Format:
        event: <type>\n
        id: <id>\n
        data: <json>\n\n
    """
    lines = []

    if event_type:
        lines.append(f"event: {event_type}")

    if event_id:
        lines.append(f"id: {event_id}")

    # Data can be multi-line, each line prefixed with "data: "
    json_str = json.dumps(data, ensure_ascii=False)
    lines.append(f"data: {json_str}")

    # End with double newline
    lines.append('')
    lines.append('')

    return '\n'.join(lines)

def generate_initialize_response(req_id=1):
    """Generate MCP initialize response"""
    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "result": {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {},
                "resources": {},
                "prompts": {}
            },
            "serverInfo": {
                "name": "mock-sse-server",
                "version": "1.0.0"
            }
        }
    }

def generate_tools_list_response(req_id=2):
    """Generate tools/list response"""
    tools = [
        {
            "name": "read_file",
            "description": "Read a file from the filesystem",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "description": "Path to the file"
                    }
                },
                "required": ["path"]
            }
        },
        {
            "name": "write_file",
            "description": "Write content to a file",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": {"type": "string"},
                    "content": {"type": "string"}
                },
                "required": ["path", "content"]
            }
        },
        {
            "name": "search_code",
            "description": "Search for code patterns",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "pattern": {"type": "string"},
                    "files": {"type": "array", "items": {"type": "string"}}
                },
                "required": ["pattern"]
            }
        }
    ]

    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "result": {
            "tools": tools
        }
    }

def generate_resources_list_response(req_id=3):
    """Generate resources/list response"""
    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "result": {
            "resources": [
                {
                    "uri": "file:///project/README.md",
                    "name": "Project README",
                    "mimeType": "text/markdown"
                },
                {
                    "uri": "file:///project/src/main.rs",
                    "name": "Main source file",
                    "mimeType": "text/rust"
                }
            ]
        }
    }

def generate_tool_call_response(req_id, tool_name):
    """Generate tool call response"""
    results = {
        "read_file": {
            "content": "# Example File\n\nThis is the file content.",
            "size": 42,
            "encoding": "utf-8"
        },
        "write_file": {
            "success": True,
            "bytesWritten": 128
        },
        "search_code": {
            "matches": [
                {"file": "src/main.rs", "line": 42, "text": "fn main() {"},
                {"file": "src/lib.rs", "line": 10, "text": "pub fn main() {"}
            ]
        }
    }

    result = results.get(tool_name, {"status": "unknown tool"})

    return {
        "jsonrpc": "2.0",
        "id": req_id,
        "result": result
    }

def generate_notification(method, params=None):
    """Generate JSON-RPC notification (no id field)"""
    notif = {
        "jsonrpc": "2.0",
        "method": method
    }
    if params:
        notif["params"] = params
    return notif

def generate_progress_notification(progress, total=100):
    """Generate progress notification"""
    return generate_notification(
        "notifications/progress",
        {
            "progress": progress,
            "total": total,
            "message": f"Processing... {progress}/{total}"
        }
    )

def generate_log_notification(level, message):
    """Generate log notification"""
    return generate_notification(
        "notifications/log",
        {
            "level": level,
            "message": message,
            "timestamp": int(time.time() * 1000)
        }
    )

@app.route('/')
def index():
    """Homepage with API info"""
    return jsonify({
        "name": "Mock MCP SSE Server",
        "version": "1.0.0",
        "transport": "SSE (Server-Sent Events)",
        "endpoints": {
            "sse": "/events",
            "health": "/health"
        },
        "config": CONFIG
    })

@app.route('/health')
def health():
    """Health check endpoint"""
    return jsonify({
        "status": "healthy",
        "timestamp": time.time()
    })

@app.route('/events')
def sse_events():
    """SSE endpoint that streams MCP messages"""
    # Capture remote_addr before entering generator context
    client_addr = request.remote_addr

    def generate():
        print(f"[SSE] Client connected: {client_addr}", file=sys.stderr)

        event_counter = 1

        # 1. Initialize response
        yield format_sse_event(
            generate_initialize_response(1),
            event_type='mcp-response',
            event_id=event_counter
        )
        event_counter += 1
        time.sleep(CONFIG['delay'])

        # 2. Tools list response
        yield format_sse_event(
            generate_tools_list_response(2),
            event_type='mcp-response',
            event_id=event_counter
        )
        event_counter += 1
        time.sleep(CONFIG['delay'])

        # 3. Resources list response
        yield format_sse_event(
            generate_resources_list_response(3),
            event_type='mcp-response',
            event_id=event_counter
        )
        event_counter += 1
        time.sleep(CONFIG['delay'])

        # 4. Generate tool call responses
        tools = ['read_file', 'write_file', 'search_code']
        for i in range(CONFIG['iterations']):
            req_id = 4 + i
            tool = random.choice(tools)

            # Tool call response
            yield format_sse_event(
                generate_tool_call_response(req_id, tool),
                event_type='mcp-response',
                event_id=event_counter
            )
            event_counter += 1

            # Progress notification
            progress = int((i + 1) / CONFIG['iterations'] * 100)
            yield format_sse_event(
                generate_progress_notification(progress),
                event_type='mcp-notification',
                event_id=event_counter
            )
            event_counter += 1

            time.sleep(CONFIG['delay'])

        # 5. Log notifications
        log_levels = ['info', 'debug', 'warning']
        log_messages = [
            'Processing completed',
            'All tools executed successfully',
            'Ready for new requests'
        ]

        for level, message in zip(log_levels, log_messages):
            yield format_sse_event(
                generate_log_notification(level, message),
                event_type='mcp-notification',
                event_id=event_counter
            )
            event_counter += 1
            time.sleep(CONFIG['delay'] * 0.5)

        # 6. Keep-alive heartbeat
        print(f"[SSE] Entering heartbeat mode", file=sys.stderr)
        while True:
            time.sleep(30)
            yield format_sse_event(
                {"type": "heartbeat", "timestamp": time.time()},
                event_type='heartbeat',
                event_id=event_counter
            )
            event_counter += 1

    return Response(
        generate(),
        mimetype='text/event-stream',
        headers={
            'Cache-Control': 'no-cache',
            'X-Accel-Buffering': 'no',  # Disable nginx buffering
            'Connection': 'keep-alive'
        }
    )

def main():
    parser = argparse.ArgumentParser(
        description='Mock MCP Server with SSE Transport'
    )
    parser.add_argument(
        '--port',
        type=int,
        default=8080,
        help='Port to listen on (default: 8080)'
    )
    parser.add_argument(
        '--iterations',
        type=int,
        default=10,
        help='Number of tool calls to generate (default: 10)'
    )
    parser.add_argument(
        '--delay',
        type=float,
        default=1.0,
        help='Delay between messages in seconds (default: 1.0)'
    )
    parser.add_argument(
        '--mode',
        choices=['realistic', 'stress', 'slow'],
        default='realistic',
        help='Message generation mode'
    )

    args = parser.parse_args()

    # Update config
    CONFIG['iterations'] = args.iterations
    CONFIG['delay'] = args.delay
    CONFIG['mode'] = args.mode

    print("="*60)
    print("Mock MCP SSE Server")
    print("="*60)
    print(f"Port:       {args.port}")
    print(f"Iterations: {args.iterations}")
    print(f"Delay:      {args.delay}s")
    print(f"Mode:       {args.mode}")
    print("")
    print(f"SSE Endpoint: http://localhost:{args.port}/events")
    print(f"Health:       http://localhost:{args.port}/health")
    print("")
    print("Test with:")
    print(f"  curl -N http://localhost:{args.port}/events")
    print(f"  curl http://localhost:{args.port}/health")
    print("="*60)
    print("")

    try:
        app.run(
            host='0.0.0.0',
            port=args.port,
            threaded=True,
            debug=False
        )
    except KeyboardInterrupt:
        print("\nShutting down...")
        sys.exit(0)

if __name__ == '__main__':
    main()
