# Reticle - MCP Protocol Inspector
# https://github.com/labterminal/reticle

# Default recipe - show available commands
default:
    @just --list

# ============================================================================
# Development
# ============================================================================

# Start the development environment (Tauri + Vite)
dev:
    #!/usr/bin/env bash
    set -euo pipefail
    ROOT="{{justfile_directory()}}"

    cleanup() {
        echo "Cleaning up..."
        pkill -f "vite.*frontend" 2>/dev/null || true
        lsof -ti:1420 | xargs kill -9 2>/dev/null || true
    }
    trap cleanup EXIT INT TERM

    echo "Starting Reticle in development mode..."

    # Kill any existing processes
    pkill -f "reticle" 2>/dev/null || true
    cleanup
    sleep 1

    # Start Vite dev server in background
    echo "Starting Vite dev server..."
    cd "$ROOT/frontend" && npm run dev &
    VITE_PID=$!

    # Wait for Vite to be ready
    echo "Waiting for Vite on http://localhost:1420..."
    for i in {1..30}; do
        if curl -s http://localhost:1420 > /dev/null 2>&1; then
            break
        fi
        sleep 1
    done

    # Start Tauri
    echo "Starting Tauri app..."
    cd "$ROOT/src-tauri" && cargo tauri dev

# Build for production
build:
    cd frontend && npm run build && cd ../src-tauri && cargo tauri build

# Check Rust code without building
check:
    cd src-tauri && cargo check

# Run clippy lints
lint:
    cd src-tauri && cargo clippy -- -D warnings

# Format code
fmt:
    cd src-tauri && cargo fmt

# Install frontend dependencies
setup:
    cd frontend && npm install

# ============================================================================
# Testing - Direct (No Proxy)
# ============================================================================

# Test MCP server directly without proxy
test-direct:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Testing MCP server directly (no proxy)..."
    echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}},"id":1}' | \
        npx -y @anthropics/mcp-server-memory 2>/dev/null | head -1

# ============================================================================
# Testing - With Proxy (mcp-sentinel)
# ============================================================================

# Build mcp-sentinel proxy
build-sentinel:
    cd target/release 2>/dev/null || cargo build --release --manifest-path src-tauri/Cargo.toml
    @echo "Sentinel built at target/release/mcp-sentinel (if available)"

# Test with proxy on default port (3001)
test-proxy port="3001":
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Testing MCP server through proxy on port {{port}}..."
    echo '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}},"id":1}' | \
        ./target/release/mcp-sentinel --port {{port}} -- npx -y @anthropics/mcp-server-memory 2>/dev/null | head -1

# Stress test with multiple sequential requests
test-stress count="100":
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Running stress test with {{count}} requests..."
    for i in $(seq 1 {{count}}); do
        echo '{"jsonrpc":"2.0","method":"ping","id":'$i'}' | \
            ./target/release/mcp-sentinel --port 3001 -- cat 2>/dev/null
    done
    echo "Stress test complete: {{count}} requests sent"

# ============================================================================
# SSE Transport Testing
# ============================================================================

# Start mock SSE server (requires Python)
sse-server port="8080":
    python3 scripts/mock-mcp-sse-server.py --port {{port}}

# Test SSE endpoint
test-sse port="8080":
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Testing SSE endpoint on port {{port}}..."
    curl -s -N "http://localhost:{{port}}/events" &
    SSE_PID=$!
    sleep 1
    curl -s -X POST "http://localhost:{{port}}/message" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}'
    sleep 1
    kill $SSE_PID 2>/dev/null || true

# ============================================================================
# Mock Servers (for development/testing)
# ============================================================================

# Run mock MCP server (stdio)
mock-server:
    python3 scripts/mock-mcp-server.py

# Run mock MCP agent
mock-agent:
    python3 scripts/mock-mcp-agent.py

# Run error test server
mock-error-server:
    python3 scripts/test-error-server.py

# ============================================================================
# Utilities
# ============================================================================

# Clean build artifacts
clean:
    rm -rf target/
    rm -rf frontend/node_modules/
    rm -rf frontend/dist/
    rm -rf src-tauri/target/

# Show project info
info:
    @echo "Reticle - MCP Protocol Inspector"
    @echo "================================="
    @echo ""
    @echo "Project structure:"
    @echo "  frontend/     - React frontend"
    @echo "  src-tauri/    - Rust backend (Tauri)"
    @echo "  scripts/      - Python test utilities"
    @echo ""
    @echo "Quick start:"
    @echo "  just setup    - Install dependencies"
    @echo "  just dev      - Start development server"
    @echo "  just build    - Build for production"

# Show current Rust/Node versions
versions:
    @echo "Rust: $(rustc --version)"
    @echo "Cargo: $(cargo --version)"
    @echo "Node: $(node --version)"
    @echo "npm: $(npm --version)"
