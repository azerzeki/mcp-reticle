# Reticle Scripts

This directory contains Python test utilities for the Reticle project.

> **Note**: Shell scripts have been replaced by the [Justfile](../Justfile). Run `just` from the project root to see available commands.

## Python Utilities

### `mock-mcp-agent.py`

Mock MCP agent that generates realistic protocol traffic.

```bash
# Standalone realistic test
python3 scripts/mock-mcp-agent.py --mode realistic --iterations 20 --verbose

# Stress test
python3 scripts/mock-mcp-agent.py --mode stress --messages 500

# Pipe to server
python3 scripts/mock-mcp-agent.py | python3 scripts/mock-mcp-server.py
```

**Features**:
- Full JSON-RPC 2.0 implementation
- MCP protocol compliance
- Tool calls (read_file, write_file, search_code, etc.)
- Resource access (file://, git:// URIs)
- Prompt interactions
- Configurable traffic patterns

---

### `mock-mcp-server.py`

Mock MCP server that responds to agent requests.

```bash
# Run standalone (reads from stdin)
python3 scripts/mock-mcp-server.py --verbose

# Pipe from agent
python3 scripts/mock-mcp-agent.py | python3 scripts/mock-mcp-server.py
```

**Features**:
- Responds to all standard MCP methods
- Realistic response times
- Supports tools, resources, and prompts
- Full error handling

---

### `mock-mcp-sse-server.py`

Mock SSE (Server-Sent Events) MCP server for testing HTTP/SSE transport.

```bash
python3 scripts/mock-mcp-sse-server.py --port 8080
```

---

### `test-error-server.py`

Test server that generates various error conditions for testing error handling.

```bash
python3 scripts/test-error-server.py
```

---

## Using Justfile Commands

The recommended way to run tests is via the Justfile:

```bash
just                  # Show all available commands
just test-direct      # Test MCP server directly
just test-proxy       # Test with proxy
just mock-server      # Run mock MCP server
just mock-agent       # Run mock MCP agent
just sse-server       # Run SSE test server
```

## Requirements

```bash
pip install -r scripts/requirements-sse.txt
```

Or for basic testing, just Python 3.8+ with standard library.
