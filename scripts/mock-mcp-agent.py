#!/usr/bin/env python3
"""
Mock MCP Agent - Simulates a realistic Model Context Protocol agent
Generates substantial traffic for testing Reticle proxy
"""

import json
import sys
import time
import random
import argparse
from typing import Dict, Any, List
from datetime import datetime

class MockMCPAgent:
    """Simulates a realistic MCP agent with various capabilities"""

    def __init__(self, agent_id: str, verbose: bool = False):
        self.agent_id = agent_id
        self.verbose = verbose
        self.request_id = 0

        # Realistic tool definitions
        self.tools = [
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

        # Realistic resource URIs
        self.resources = [
            "file:///workspace/src/main.rs",
            "file:///workspace/src/lib.rs",
            "file:///workspace/Cargo.toml",
            "file:///workspace/README.md",
            "git://repo/commit/abc123",
            "https://api.example.com/data"
        ]

        # Realistic prompts
        self.prompts = [
            {
                "name": "code_review",
                "description": "Review code for best practices",
                "arguments": [
                    {"name": "code", "description": "Code to review", "required": True}
                ]
            },
            {
                "name": "debug_error",
                "description": "Help debug an error message",
                "arguments": [
                    {"name": "error", "description": "Error message", "required": True}
                ]
            }
        ]

    def log(self, message: str):
        """Log to stderr for debugging"""
        if self.verbose:
            print(f"[{self.agent_id}] {message}", file=sys.stderr)

    def next_id(self) -> str:
        """Generate next request ID"""
        self.request_id += 1
        return f"{self.agent_id}-{self.request_id}"

    def send_request(self, method: str, params: Dict[str, Any] = None):
        """Send JSON-RPC request"""
        request = {
            "jsonrpc": "2.0",
            "id": self.next_id(),
            "method": method
        }
        if params:
            request["params"] = params

        self.log(f"→ {method}")
        print(json.dumps(request), flush=True)
        return request["id"]

    def send_notification(self, method: str, params: Dict[str, Any] = None):
        """Send JSON-RPC notification (no response expected)"""
        notification = {
            "jsonrpc": "2.0",
            "method": method
        }
        if params:
            notification["params"] = params

        self.log(f"→ [notification] {method}")
        print(json.dumps(notification), flush=True)

    def read_response(self) -> Dict[str, Any]:
        """Read JSON-RPC response"""
        line = sys.stdin.readline()
        if not line:
            return None

        try:
            response = json.loads(line)
            self.log(f"← {response.get('id', 'notification')}")
            return response
        except json.JSONDecodeError:
            self.log(f"Warning: Invalid JSON response: {line}")
            return None

    def initialize(self):
        """Send initialize request"""
        self.log("Initializing MCP session...")
        request_id = self.send_request("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {
                "tools": {"listChanged": True},
                "resources": {"subscribe": True, "listChanged": True},
                "prompts": {"listChanged": True}
            },
            "clientInfo": {
                "name": f"mock-agent-{self.agent_id}",
                "version": "1.0.0"
            }
        })

        response = self.read_response()
        if response:
            server_name = response.get('result', {}).get('serverInfo', {}).get('name', 'Unknown')
            self.log(f"Initialized: {server_name}")
        else:
            self.log("Warning: No response received for initialize")

        # Send initialized notification
        self.send_notification("notifications/initialized")
        return response

    def list_tools(self):
        """List available tools"""
        self.send_request("tools/list")
        return self.read_response()

    def call_tool(self, tool_name: str, arguments: Dict[str, Any]):
        """Call a tool"""
        self.send_request("tools/call", {
            "name": tool_name,
            "arguments": arguments
        })
        return self.read_response()

    def list_resources(self):
        """List available resources"""
        self.send_request("resources/list")
        return self.read_response()

    def read_resource(self, uri: str):
        """Read a resource"""
        self.send_request("resources/read", {"uri": uri})
        return self.read_response()

    def list_prompts(self):
        """List available prompts"""
        self.send_request("prompts/list")
        return self.read_response()

    def get_prompt(self, prompt_name: str, arguments: Dict[str, Any] = None):
        """Get a prompt"""
        params = {"name": prompt_name}
        if arguments:
            params["arguments"] = arguments
        self.send_request("prompts/get", params)
        return self.read_response()

    def simulate_realistic_workflow(self, iterations: int = 10, delay: float = 0.1):
        """Simulate realistic agent workflow with various operations"""
        self.log(f"Starting realistic workflow: {iterations} iterations")

        workflows = [
            self.workflow_code_analysis,
            self.workflow_file_operations,
            self.workflow_resource_access,
            self.workflow_prompt_interaction,
            self.workflow_mixed_operations
        ]

        for i in range(iterations):
            workflow = random.choice(workflows)
            self.log(f"Iteration {i+1}/{iterations}: {workflow.__name__}")
            workflow()
            time.sleep(delay)

    def workflow_code_analysis(self):
        """Simulate code analysis workflow"""
        # List tools
        self.list_tools()
        time.sleep(0.05)

        # Search for code patterns
        self.call_tool("search_code", {
            "pattern": "async fn",
            "file_type": "rs"
        })
        time.sleep(0.05)

        # Read file
        self.call_tool("read_file", {
            "path": "/workspace/src/main.rs"
        })
        time.sleep(0.05)

    def workflow_file_operations(self):
        """Simulate file operations workflow"""
        # List directory
        self.call_tool("list_directory", {
            "path": "/workspace/src"
        })
        time.sleep(0.05)

        # Read file
        self.call_tool("read_file", {
            "path": "/workspace/Cargo.toml"
        })
        time.sleep(0.05)

        # Write file
        self.call_tool("write_file", {
            "path": "/tmp/test.txt",
            "content": f"Test at {datetime.now().isoformat()}"
        })
        time.sleep(0.05)

    def workflow_resource_access(self):
        """Simulate resource access workflow"""
        # List resources
        self.list_resources()
        time.sleep(0.05)

        # Read random resources
        for _ in range(random.randint(1, 3)):
            uri = random.choice(self.resources)
            self.read_resource(uri)
            time.sleep(0.05)

    def workflow_prompt_interaction(self):
        """Simulate prompt interaction workflow"""
        # List prompts
        self.list_prompts()
        time.sleep(0.05)

        # Get prompt
        self.get_prompt("code_review", {
            "code": "fn main() { println!(\"Hello\"); }"
        })
        time.sleep(0.05)

    def workflow_mixed_operations(self):
        """Simulate mixed operations"""
        operations = [
            lambda: self.list_tools(),
            lambda: self.list_resources(),
            lambda: self.list_prompts(),
            lambda: self.call_tool("read_file", {"path": "/workspace/README.md"}),
            lambda: self.read_resource(random.choice(self.resources))
        ]

        for _ in range(random.randint(3, 6)):
            op = random.choice(operations)
            op()
            time.sleep(0.05)

    def run_stress_test(self, messages: int = 100, burst_size: int = 10):
        """Generate high-volume traffic for stress testing"""
        self.log(f"Starting stress test: {messages} messages in bursts of {burst_size}")

        for i in range(0, messages, burst_size):
            # Send burst of requests
            for j in range(min(burst_size, messages - i)):
                operation = random.choice([
                    lambda: self.list_tools(),
                    lambda: self.call_tool("read_file", {"path": f"/file{j}.txt"}),
                    lambda: self.list_resources(),
                    lambda: self.read_resource(f"file:///workspace/file{j}.rs")
                ])
                operation()

            # Small delay between bursts
            time.sleep(0.01)

            if (i + burst_size) % 50 == 0:
                self.log(f"Progress: {i + burst_size}/{messages} messages sent")

def main():
    parser = argparse.ArgumentParser(description="Mock MCP Agent for testing")
    parser.add_argument("--id", default="test-agent", help="Agent ID")
    parser.add_argument("--mode", choices=["realistic", "stress"], default="realistic",
                       help="Test mode: realistic workflow or stress test")
    parser.add_argument("--iterations", type=int, default=20,
                       help="Number of workflow iterations (realistic mode)")
    parser.add_argument("--messages", type=int, default=200,
                       help="Number of messages to send (stress mode)")
    parser.add_argument("--burst-size", type=int, default=10,
                       help="Burst size for stress test")
    parser.add_argument("--delay", type=float, default=0.1,
                       help="Delay between operations (seconds)")
    parser.add_argument("--verbose", action="store_true",
                       help="Enable verbose logging to stderr")

    args = parser.parse_args()

    agent = MockMCPAgent(args.id, verbose=args.verbose)

    try:
        # Initialize
        agent.initialize()
        time.sleep(0.1)

        # Run selected mode
        if args.mode == "realistic":
            agent.simulate_realistic_workflow(args.iterations, args.delay)
        else:  # stress
            agent.run_stress_test(args.messages, args.burst_size)

        agent.log("Test completed successfully")

    except KeyboardInterrupt:
        agent.log("Interrupted by user")
    except Exception as e:
        agent.log(f"Error: {e}")
        raise

if __name__ == "__main__":
    main()
