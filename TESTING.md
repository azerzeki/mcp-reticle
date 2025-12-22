# MCP Sentinel Testing Guide

Complete guide for testing MCP Sentinel with stdio transport, HTTP/SSE transport, mock servers, and real MCP servers.

## Table of Contents

1. [Quick Start Testing](#quick-start-testing)
2. [stdio Transport Testing](#stdio-transport-testing)
3. [HTTP/SSE Transport Testing](#httpsse-transport-testing)
4. [Demo Mode](#demo-mode)
5. [Testing Scenarios](#testing-scenarios)
6. [Viewing Logs](#viewing-logs)
7. [Troubleshooting](#troubleshooting)
8. [Advanced Testing](#advanced-testing)

## Quick Start Testing

### 30-Second Test (Fastest Way)

```bash
# 1. Start the desktop app
./scripts/dev.sh

# 2. In the UI - Configure Real Proxy:
#    Command: bash
#    Args: scripts/run-mock-pair.sh --iterations 30

# 3. Click "Start"

# 4. Watch ~225 messages appear in real-time!
```

This is the fastest way to verify MCP Sentinel is working correctly.

## stdio Transport Testing

### Overview

stdio transport uses stdin/stdout pipes to communicate with process-based MCP servers. This is the primary transport mode and is fully implemented.

### Test 1: Basic Mock Test

**Purpose:** Verify basic functionality with 10 iterations

```bash
# Start app
./scripts/dev.sh

# In UI, configure:
Command: bash
Args: scripts/run-mock-pair.sh --iterations 10

# Click "Start"
```

**Expected Output:**
- ~75 messages total
- Messages include: initialize, tools/list, tools/call responses
- All messages appear in real-time
- Completes in ~5-10 seconds

### Test 2: Stress Test

**Purpose:** Test high-volume message handling

```bash
Command: bash
Args: scripts/run-mock-pair.sh --iterations 100 --delay 0.05
```

**Expected Output:**
- ~750 messages total
- UI remains responsive
- Virtualization keeps rendering smooth
- Completes in ~15-20 seconds

### Test 3: Slow Watch Test

**Purpose:** Detailed observation of each message

```bash
Command: bash
Args: scripts/run-mock-pair.sh --iterations 5 --delay 2.0
```

**Expected Output:**
- ~37 messages total
- 2-second delay between iterations
- Easy to observe each message as it arrives
- Good for understanding message flow

### Test 4: Workflow-Specific Tests

Test specific MCP workflows:

```bash
# Code analysis workflow
Args: scripts/run-mock-pair.sh --mode code-analysis --iterations 20

# File operations workflow
Args: scripts/run-mock-pair.sh --mode file-operations --iterations 20

# Resource access workflow
Args: scripts/run-mock-pair.sh --mode resource-access --iterations 20

# Prompt interaction workflow
Args: scripts/run-mock-pair.sh --mode prompt-interaction --iterations 20
```

### Test 5: Real MCP Server

**Purpose:** Test with actual MCP server implementation

```bash
# Python MCP server
Command: python3
Args: /path/to/your/mcp_server.py

# Node.js MCP server
Command: node
Args: /path/to/your/server.js

# Official filesystem server
Command: npx
Args: -y @modelcontextprotocol/server-filesystem /workspace
```

**What You'll See:**
- Real traffic from your client to server
- Actual protocol implementation
- Real-world timing and performance

## HTTP/SSE Transport Testing

### Overview

HTTP/SSE transport is designed for web-based and cloud-hosted MCP servers. The backend implementation is complete but needs frontend UI integration.

### Prerequisites

```bash
# Install Python dependencies for mock SSE server
pip3 install flask
```

### Test 1: Mock SSE Server Direct Connection

**Purpose:** Verify mock SSE server works independently

```bash
# Terminal 1: Start mock SSE server
python3 scripts/mock-mcp-sse-server.py --port 8080 --iterations 10 --delay 1.0

# Terminal 2: Test with curl
curl -N http://localhost:8080/events
```

**Expected Output:**
```
data: {"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05"...}}

data: {"jsonrpc":"2.0","id":2,"result":{"tools":[...]}}

data: {"jsonrpc":"2.0","method":"notifications/progress","params":{...}}
```

### Test 2: SSE Proxy via Browser Console

**Purpose:** Test SSE proxy backend (UI not yet integrated)

```bash
# Terminal 1: Start mock SSE server
python3 scripts/mock-mcp-sse-server.py --port 8080

# Terminal 2: Start MCP Sentinel
./scripts/dev.sh

# In browser console (F12):
await window.__TAURI__.invoke('start_proxy_v2', {
  transportConfig: {
    type: 'http',
    http: {
      serverUrl: 'http://localhost:8080',
      proxyPort: 3001
    }
  }
})

# Terminal 3: Connect to proxy
curl -N http://localhost:3001/events
```

**Expected Output:**
- SSE events proxied through MCP Sentinel
- Messages appear in UI (via log-event emission)
- Both terminals show same events

### Test 3: Health Check

```bash
# Check SSE server health
curl http://localhost:8080/health

# Check SSE proxy health (after starting)
curl http://localhost:3001/health
```

### SSE Proxy Configuration

When UI is integrated, configure:
- **Transport Type:** HTTP/SSE
- **Server URL:** http://localhost:8080
- **Proxy Port:** 3001

## Demo Mode

### Overview

Demo mode loads pre-generated MCP conversation data without requiring any external processes.

### How to Use

```bash
# Start app
./scripts/dev.sh

# In UI:
1. Click "Load Demo Data" button
# OR
2. Configure with command "demo" and click Start
```

### What's Included

- ~225 pre-loaded messages
- Various message types:
  - initialize request/response
  - tools/list request/response
  - tools/call request/response
  - notifications
  - errors
- Good for:
  - UI development
  - Feature testing
  - Demos and presentations

## Testing Scenarios

### Scenario 1: First-Time Setup Test

**Goal:** Verify clean installation works

```bash
# 1. Clone and build
git clone <repo-url>
cd mcp-sentinel
cargo build --manifest-path src-tauri/Cargo.toml

# 2. Start app
./scripts/dev.sh

# 3. Load demo data
# Click "Load Demo Data"

# 4. Test mock pair
# Configure: bash scripts/run-mock-pair.sh --iterations 10
```

**Success Criteria:**
- App starts without errors
- Demo data loads and displays
- Mock pair generates messages
- UI is responsive

### Scenario 2: Performance Test

**Goal:** Verify handling of high message volume

```bash
# Test with 1000 iterations
Command: bash
Args: scripts/run-mock-pair.sh --iterations 1000 --delay 0.01
```

**Monitor:**
- Memory usage (should stay reasonable)
- UI responsiveness (should remain smooth)
- Message rendering (virtualization should work)
- No crashes or freezes

### Scenario 3: Long-Running Session

**Goal:** Test stability over time

```bash
# Run for extended period
Command: bash
Args: scripts/run-mock-pair.sh --iterations 100 --delay 5.0
```

**Monitor:**
- Memory leaks (check Activity Monitor/Task Manager)
- UI remains responsive
- Events continue to emit correctly
- No degradation over time

### Scenario 4: Filter and Search Test

**Goal:** Verify filtering functionality

```bash
# Generate diverse messages
Command: bash
Args: scripts/run-mock-pair.sh --iterations 30

# Then in UI:
1. Filter by direction (In/Out)
2. Filter by method (initialize, tools/list, etc.)
3. Search for specific text
4. Clear filters
```

**Success Criteria:**
- Filters work immediately
- Message count updates correctly
- Search highlights matches
- Clear filters restores full list

### Scenario 5: Inspector Test

**Goal:** Verify JSON inspection works

```bash
# Generate messages
Command: bash
Args: scripts/run-mock-pair.sh --iterations 10

# Then in UI:
1. Click on various messages
2. Inspect JSON in right panel
3. Use copy button
4. Toggle expand/collapse
```

**Success Criteria:**
- JSON displays with syntax highlighting
- Copy works correctly
- Monaco editor loads properly
- Large JSON objects display well

## Viewing Logs

### UI Logs

Messages appear in three places:

1. **LogStream Panel (Center)**
   - Real-time message cards
   - Color-coded by direction
   - Virtualized for performance

2. **Inspector Panel (Right)**
   - Detailed JSON view
   - Syntax highlighting
   - Copy functionality

3. **Metrics Sidebar (Left)**
   - Total message count
   - Direction breakdown
   - Top methods

### Terminal Logs

Enable debug logging:

```bash
# Set log level
export RUST_LOG=debug

# Start app with logging
./scripts/dev.sh 2>&1 | tee /tmp/mcp-sentinel-debug.log
```

**View logs in real-time:**
```bash
# All logs
tail -f /tmp/mcp-sentinel-debug.log

# Just proxy events
tail -f /tmp/mcp-sentinel-debug.log | grep "proxy"

# Just log events
tail -f /tmp/mcp-sentinel-debug.log | grep "log-event"

# Colorized logs (if watch-messages.sh available)
./scripts/watch-messages.sh
```

### Log Patterns to Look For

**Normal Operation:**
```
DEBUG mcp_sentinel_gui::core::proxy: Proxy loop started for session session-XXXXX
DEBUG mcp_sentinel_gui::core::proxy: Out: {"jsonrpc":"2.0","id":"test-agent-1"...}
DEBUG mcp_sentinel_gui::core::proxy: Emitted log-event: msg-123
```

**Errors:**
```
ERROR mcp_sentinel_gui::core::proxy: Failed to parse JSON: ...
ERROR mcp_sentinel_gui::commands::proxy: Failed to spawn process: ...
```

## Troubleshooting

### Problem: No Messages Appearing in UI

**Symptoms:**
- Proxy starts (green indicator)
- No messages in LogStream
- Empty metrics

**Diagnosis Steps:**

1. **Check if events are being emitted:**
   ```bash
   grep "Emitted log-event" /tmp/mcp-sentinel-debug.log
   ```
   If no results, backend is not emitting events.

2. **Check if proxy is reading data:**
   ```bash
   grep "Out:" /tmp/mcp-sentinel-debug.log
   ```
   If no results, child process is not producing output.

3. **Check if child process started:**
   ```bash
   grep "Spawning" /tmp/mcp-sentinel-debug.log
   ```

**Common Causes & Solutions:**

**Cause 1: Using mock server alone (without agent)**
```bash
# WRONG - server has no input
Command: python3
Args: scripts/mock-mcp-server.py

# CORRECT - agent pipes to server
Command: bash
Args: scripts/run-mock-pair.sh --iterations 30
```

**Cause 2: Wrong working directory**
- The fix is already in place (src-tauri/src/commands/proxy.rs)
- If still an issue, check logs for "current_dir"

**Cause 3: Python output buffering**
- Make sure scripts use `-u` flag
- The fix is in run-mock-pair.sh

**Cause 4: Process exits too quickly**
- Add more iterations: `--iterations 30`
- Add delay: `--delay 1.0`

### Problem: Proxy Won't Start

**Symptoms:**
- Error toast appears
- Red status indicator
- Error in terminal logs

**Diagnosis:**

1. **Check error message in terminal:**
   ```bash
   grep ERROR /tmp/mcp-sentinel-debug.log | tail -5
   ```

2. **Common errors:**

**Error: "Command not found"**
```bash
# Check command exists
which python3
which bash

# Use full path if needed
Command: /usr/bin/python3
```

**Error: "Permission denied"**
```bash
# Make script executable
chmod +x scripts/run-mock-pair.sh

# Or use bash explicitly
Command: bash
Args: scripts/run-mock-pair.sh --iterations 30
```

**Error: "Port already in use" (HTTP/SSE mode)**
```bash
# Check what's using the port
lsof -ti:3001

# Kill the process
lsof -ti:3001 | xargs kill -9

# Or use different port in config
```

### Problem: UI Not Updating

**Symptoms:**
- Messages in terminal logs
- Events being emitted
- But UI stays empty

**Diagnosis:**

1. **Check browser console (F12):**
   ```javascript
   // Look for errors in console
   // Check if listeners are registered
   ```

2. **Check event listener:**
   Open devtools and verify:
   ```
   Setting up Tauri event listeners...
   Event listener registered successfully
   ```

3. **Force refresh:**
   - Click "Clear Logs"
   - Restart proxy
   - Refresh browser (Cmd+R / Ctrl+R)

### Problem: Messages Not Filtered Correctly

**Symptoms:**
- Filter applied but wrong messages shown
- Search not working

**Solution:**

1. **Clear filters first:**
   - Click "Clear Filters" button
   - Verify all messages appear

2. **Apply filter one at a time:**
   - Direction filter
   - Method filter
   - Search filter

3. **Check case sensitivity:**
   - Search is case-insensitive
   - But should match partial strings

### Problem: Performance Issues / UI Lag

**Symptoms:**
- Slow scrolling
- Delayed updates
- High memory usage

**Solutions:**

1. **Reduce message volume:**
   ```bash
   # Use fewer iterations
   --iterations 10

   # Add delay between messages
   --delay 0.5
   ```

2. **Clear old logs:**
   - Click "Clear Logs" button
   - Restart session

3. **Check virtualization:**
   - Virtualization should be enabled by default
   - Only ~30 rows rendered at a time

### Problem: SSE Proxy Not Working

**Symptoms:**
- Can't connect to http://localhost:3001
- No SSE events received

**Diagnosis:**

1. **Check if SSE proxy is running:**
   ```bash
   curl http://localhost:3001/health
   ```
   Should return: "SSE Proxy is healthy"

2. **Check if real MCP server is running:**
   ```bash
   curl http://localhost:8080/health
   ```

3. **Check terminal logs:**
   ```bash
   grep "SSE PROXY" /tmp/mcp-sentinel-debug.log
   ```

**Solutions:**

1. **Make sure mock SSE server is running:**
   ```bash
   python3 scripts/mock-mcp-sse-server.py --port 8080
   ```

2. **Check firewall:**
   - Allow connections on ports 3001 and 8080

3. **Use correct URL:**
   ```bash
   # Connect to PROXY, not server directly
   curl -N http://localhost:3001/events  # Correct
   curl -N http://localhost:8080/events  # Wrong (bypasses proxy)
   ```

## Advanced Testing

### Custom Test Scripts

Create your own test MCP client:

```python
#!/usr/bin/env python3
"""custom_mcp_client.py"""
import json
import sys

# Send initialize request
request = {
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {
        "protocolVersion": "2024-11-05",
        "clientInfo": {"name": "custom-client", "version": "1.0.0"}
    }
}
print(json.dumps(request), flush=True)

# Read response
response = json.loads(sys.stdin.readline())
print(f"Got response: {response}", file=sys.stderr)
```

Test with:
```bash
Command: python3
Args: custom_mcp_client.py | python3 scripts/mock-mcp-server.py
```

### Testing with Real Clients

1. **Claude Desktop:**
   - Configure claude_desktop_config.json
   - Point to MCP Sentinel as proxy
   - Monitor all Claude ↔ Server traffic

2. **MCP Inspector:**
   - Use MCP Inspector as client
   - MCP Sentinel as monitoring layer
   - Compare both tools' views

### Automated Testing

Run verification script:

```bash
./scripts/verify-project.sh
```

Checks:
- Build system
- Project structure
- Critical files
- Test scripts
- Documentation
- Code quality

### Performance Benchmarking

```bash
# Measure message throughput
time bash scripts/run-mock-pair.sh --iterations 1000 --delay 0

# Monitor memory usage
# macOS:
top -pid $(pgrep -f mcp-sentinel)

# Linux:
htop -p $(pgrep -f mcp-sentinel)
```

### Debugging Tips

1. **Enable verbose logging:**
   ```bash
   RUST_LOG=trace ./scripts/dev.sh
   ```

2. **Use browser devtools:**
   - F12 to open devtools
   - Check Console for JS errors
   - Check Network for failed requests
   - Check Application > Local Storage

3. **Test components in isolation:**
   ```bash
   # Test mock agent alone
   python3 -u scripts/mock-mcp-agent.py --iterations 5

   # Test mock server alone (won't produce output without input)
   echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | \
   python3 -u scripts/mock-mcp-server.py
   ```

4. **Compare with working configuration:**
   - Always keep a known-good configuration
   - Test against it when debugging issues

## Summary

### Quick Reference

**Fast test:**
```bash
./scripts/dev.sh
# UI: bash scripts/run-mock-pair.sh --iterations 30
```

**Demo mode:**
```bash
./scripts/dev.sh
# UI: Click "Load Demo Data"
```

**Real server:**
```bash
./scripts/dev.sh
# UI: python3 /path/to/your/server.py
```

**Check logs:**
```bash
tail -f /tmp/mcp-sentinel-debug.log | grep -E "log-event|ERROR"
```

### Common Commands

```bash
# Start development
./scripts/dev.sh

# Run verification
./scripts/verify-project.sh

# View logs
tail -f /tmp/mcp-sentinel-debug.log

# Test SSE server
python3 scripts/mock-mcp-sse-server.py --port 8080

# Test components
python3 -u scripts/mock-mcp-agent.py --iterations 5
```

### Next Steps

1. Complete HTTP/SSE UI integration
2. Add automated integration tests
3. Create more test scenarios
4. Document real-world server configurations
5. Build performance profiling tools

For implementation details, see **CURRENT_WORK.md**.
For AI assistant context, see **FOR_CLAUDE.md**.
