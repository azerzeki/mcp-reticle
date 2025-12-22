#!/usr/bin/env python3
"""
Mock MCP Server - Responds to MCP agent requests
Simulates a realistic Model Context Protocol server
"""

import json
import sys
import time
import random
from typing import Dict, Any

class MockMCPServer:
    """Simulates a realistic MCP server responding to agent requests"""

    def __init__(self, server_name: str = "mock-server", verbose: bool = False):
        self.server_name = server_name
        self.verbose = verbose
        self.initialized = False

    def log(self, message: str):
        """Log to stderr for debugging"""
        if self.verbose:
            print(f"[{self.server_name}] {message}", file=sys.stderr)

    def send_response(self, request_id: str, result: Any):
        """Send JSON-RPC response"""
        response = {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": result
        }
        self.log(f"→ Response to {request_id}")
        print(json.dumps(response), flush=True)

    def send_error(self, request_id: str, code: int, message: str):
        """Send JSON-RPC error response"""
        response = {
            "jsonrpc": "2.0",
            "id": request_id,
            "error": {
                "code": code,
                "message": message
            }
        }
        self.log(f"→ Error to {request_id}: {message}")
        print(json.dumps(response), flush=True)

    def handle_initialize(self, request_id: str, params: Dict[str, Any]):
        """Handle initialize request"""
        self.log("Initializing...")
        self.initialized = True

        result = {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {"listChanged": True},
                "resources": {"subscribe": True, "listChanged": True},
                "prompts": {"listChanged": True}
            },
            "serverInfo": {
                "name": self.server_name,
                "version": "1.0.0"
            }
        }
        self.send_response(request_id, result)

    def handle_tools_list(self, request_id: str):
        """Handle tools/list request"""
        tools = [
            {
                "name": "read_file",
                "description": "Read contents of a file",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
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
                "name": "list_directory",
                "description": "List files in a directory",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"}
                    },
                    "required": ["path"]
                }
            },
            {
                "name": "execute_command",
                "description": "Execute a shell command",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "command": {"type": "string"}
                    },
                    "required": ["command"]
                }
            },
            {
                "name": "search_code",
                "description": "Search for code patterns",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "pattern": {"type": "string"},
                        "file_type": {"type": "string"}
                    },
                    "required": ["pattern"]
                }
            }
        ]
        self.send_response(request_id, {"tools": tools})

    def handle_tools_call(self, request_id: str, params: Dict[str, Any]):
        """Handle tools/call request"""
        tool_name = params.get("name")
        arguments = params.get("arguments", {})

        self.log(f"Calling tool: {tool_name}")

        # Simulate tool execution with realistic responses
        if tool_name == "read_file":
            content = f"// Mock file content for {arguments.get('path')}\n"
            content += "fn main() {\n"
            content += "    println!(\"Hello, world!\");\n"
            content += "}\n"
            result = {
                "content": [
                    {
                        "type": "text",
                        "text": content
                    }
                ]
            }
        elif tool_name == "write_file":
            result = {
                "content": [
                    {
                        "type": "text",
                        "text": f"Successfully wrote to {arguments.get('path')}"
                    }
                ]
            }
        elif tool_name == "list_directory":
            files = ["main.rs", "lib.rs", "mod.rs", "utils.rs"]
            result = {
                "content": [
                    {
                        "type": "text",
                        "text": "\n".join(files)
                    }
                ]
            }
        elif tool_name == "execute_command":
            result = {
                "content": [
                    {
                        "type": "text",
                        "text": f"Executed: {arguments.get('command')}\nOutput: Success"
                    }
                ]
            }
        elif tool_name == "search_code":
            matches = [
                "src/main.rs:12: async fn process() {",
                "src/lib.rs:45: async fn handle_request() {"
            ]
            result = {
                "content": [
                    {
                        "type": "text",
                        "text": "\n".join(matches)
                    }
                ]
            }
        else:
            self.send_error(request_id, -32601, f"Tool not found: {tool_name}")
            return

        # Simulate processing time
        time.sleep(random.uniform(0.01, 0.05))
        self.send_response(request_id, result)

    def handle_resources_list(self, request_id: str):
        """Handle resources/list request"""
        resources = [
            {
                "uri": "file:///workspace/src/main.rs",
                "name": "main.rs",
                "mimeType": "text/x-rust"
            },
            {
                "uri": "file:///workspace/src/lib.rs",
                "name": "lib.rs",
                "mimeType": "text/x-rust"
            },
            {
                "uri": "file:///workspace/Cargo.toml",
                "name": "Cargo.toml",
                "mimeType": "text/x-toml"
            },
            {
                "uri": "file:///workspace/README.md",
                "name": "README.md",
                "mimeType": "text/markdown"
            }
        ]
        self.send_response(request_id, {"resources": resources})

    def handle_resources_read(self, request_id: str, params: Dict[str, Any]):
        """Handle resources/read request"""
        uri = params.get("uri")
        self.log(f"Reading resource: {uri}")

        content = f"# Content of {uri}\n\nThis is mock content for testing."
        result = {
            "contents": [
                {
                    "uri": uri,
                    "mimeType": "text/plain",
                    "text": content
                }
            ]
        }

        time.sleep(random.uniform(0.01, 0.03))
        self.send_response(request_id, result)

    def handle_prompts_list(self, request_id: str):
        """Handle prompts/list request"""
        prompts = [
            {
                "name": "code_review",
                "description": "Review code for best practices",
                "arguments": [
                    {
                        "name": "code",
                        "description": "Code to review",
                        "required": True
                    }
                ]
            },
            {
                "name": "debug_error",
                "description": "Help debug an error message",
                "arguments": [
                    {
                        "name": "error",
                        "description": "Error message",
                        "required": True
                    }
                ]
            }
        ]
        self.send_response(request_id, {"prompts": prompts})

    def handle_prompts_get(self, request_id: str, params: Dict[str, Any]):
        """Handle prompts/get request"""
        prompt_name = params.get("name")
        arguments = params.get("arguments", {})

        self.log(f"Getting prompt: {prompt_name}")

        messages = [
            {
                "role": "user",
                "content": {
                    "type": "text",
                    "text": f"Please {prompt_name.replace('_', ' ')} the following:\n{json.dumps(arguments, indent=2)}"
                }
            }
        ]

        result = {
            "description": f"Generated prompt for {prompt_name}",
            "messages": messages
        }

        time.sleep(random.uniform(0.01, 0.03))
        self.send_response(request_id, result)

    def handle_request(self, request: Dict[str, Any]):
        """Handle incoming JSON-RPC request"""
        method = request.get("method")
        params = request.get("params", {})
        request_id = request.get("id")

        self.log(f"← {method} (id: {request_id})")

        # Handle notifications (no response needed)
        if request_id is None:
            if method == "notifications/initialized":
                self.log("Client initialized")
            return

        # Route to appropriate handler
        if method == "initialize":
            self.handle_initialize(request_id, params)
        elif method == "tools/list":
            self.handle_tools_list(request_id)
        elif method == "tools/call":
            self.handle_tools_call(request_id, params)
        elif method == "resources/list":
            self.handle_resources_list(request_id)
        elif method == "resources/read":
            self.handle_resources_read(request_id, params)
        elif method == "prompts/list":
            self.handle_prompts_list(request_id)
        elif method == "prompts/get":
            self.handle_prompts_get(request_id, params)
        else:
            self.send_error(request_id, -32601, f"Method not found: {method}")

    def run(self):
        """Run the server, processing requests from stdin"""
        self.log(f"Server {self.server_name} started, waiting for requests...")

        try:
            while True:
                line = sys.stdin.readline()
                if not line:
                    break

                try:
                    request = json.loads(line)
                    self.handle_request(request)
                except json.JSONDecodeError as e:
                    self.log(f"Invalid JSON: {e}")
                except Exception as e:
                    self.log(f"Error handling request: {e}")
                    if "id" in request:
                        self.send_error(request["id"], -32603, str(e))

        except KeyboardInterrupt:
            self.log("Server stopped by user")
        except Exception as e:
            self.log(f"Server error: {e}")
            raise

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Mock MCP Server")
    parser.add_argument("--name", default="mock-server", help="Server name")
    parser.add_argument("--verbose", action="store_true", help="Enable verbose logging")

    args = parser.parse_args()

    server = MockMCPServer(args.name, args.verbose)
    server.run()
