//! Reticle CLI
//!
//! Command-line interface for the Reticle MCP debugging proxy.
//! This binary wraps MCP server processes and forwards all traffic
//! while logging messages for debugging.
//!
//! # Architecture: Hub-and-Spoke
//!
//! Reticle uses a distributed sidecar pattern:
//! - **Hub**: The Reticle GUI daemon listens on `/tmp/reticle.sock`
//! - **Spoke**: Each `reticle run` instance wraps an MCP server and streams
//!   telemetry to the Hub
//!
//! This allows monitoring multiple MCP servers (e.g., GitHub, Postgres, Filesystem)
//! from a single unified dashboard.
//!
//! # Usage
//!
//! ```bash
//! # In claude_desktop_config.json:
//! {
//!   "mcpServers": {
//!     "github": {
//!       "command": "reticle",
//!       "args": ["run", "--name", "github", "--", "npx", "-y", "@modelcontextprotocol/server-github"]
//!     },
//!     "filesystem": {
//!       "command": "reticle",
//!       "args": ["run", "--name", "filesystem", "--", "npx", "-y", "@anthropic/mcp-server-filesystem", "/path"]
//!     }
//!   }
//! }
//! ```
//!
//! # Fail-Open Design
//!
//! The CLI wrapper is designed to "fail open" - if the Hub (GUI) is not running,
//! the wrapper continues to proxy traffic normally. Observability is optional;
//! agent functionality is never degraded.

use clap::{Parser, Subcommand};
use reticle_core::events::{InjectReceiver, NoOpEventSink, StdoutEventSink, UnixSocketEventSink};
use std::process::ExitCode;
use tracing_subscriber::EnvFilter;

mod http_proxy;
mod proxy;

/// Reticle - The Wireshark for the Model Context Protocol
///
/// A high-performance observability proxy for MCP servers.
/// Intercepts JSON-RPC traffic between hosts (Claude, IDEs) and MCP servers.
#[derive(Parser, Debug)]
#[command(name = "reticle")]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    // Legacy flags for backwards compatibility (deprecated)
    /// [DEPRECATED] Use 'reticle run' instead
    #[arg(short, long, hide = true)]
    port: Option<u16>,

    /// [DEPRECATED] Use 'reticle run' instead
    #[arg(short, long, hide = true)]
    format: Option<String>,

    /// [DEPRECATED] Use 'reticle run --name' instead
    #[arg(short, long, hide = true)]
    name: Option<String>,

    /// [DEPRECATED] Now default behavior in 'run' mode
    #[arg(long, hide = true)]
    gui: bool,

    /// Legacy: command to run (use 'reticle run -- <command>' instead)
    #[arg(trailing_var_arg = true)]
    command_args: Vec<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Wrap an MCP server and stream telemetry to the Reticle Hub
    ///
    /// This is the primary mode for use with Claude Desktop and other MCP hosts.
    /// The wrapper connects to the Hub's Unix socket and streams all JSON-RPC
    /// traffic for visualization in the dashboard.
    ///
    /// If the Hub is not running, the wrapper operates in "fail-open" mode -
    /// it continues to proxy traffic normally, just without telemetry.
    Run {
        /// Server name for identification in the dashboard
        ///
        /// If not provided, extracted from the command name
        #[arg(short, long)]
        name: Option<String>,

        /// Socket path for Hub connection
        ///
        /// Defaults to /tmp/reticle.sock (or RETICLE_SOCKET env var)
        #[arg(long, env = "RETICLE_SOCKET")]
        socket: Option<String>,

        /// Disable telemetry (pure proxy mode)
        ///
        /// Use this if you want the wrapper for process management
        /// but don't need observability
        #[arg(long)]
        no_telemetry: bool,

        /// The MCP server command and arguments
        #[arg(trailing_var_arg = true, required = true)]
        command: Vec<String>,
    },

    /// Output MCP traffic to stderr (standalone mode, no Hub)
    ///
    /// Useful for quick debugging without running the full GUI.
    Log {
        /// Server name for identification
        #[arg(short, long)]
        name: Option<String>,

        /// Output format
        #[arg(short, long, default_value = "text")]
        format: LogFormat,

        /// The MCP server command and arguments
        #[arg(trailing_var_arg = true, required = true)]
        command: Vec<String>,
    },

    /// HTTP reverse proxy for remote MCP servers
    ///
    /// Creates an HTTP proxy that intercepts traffic to a remote MCP server
    /// (SSE, Streamable HTTP) and streams telemetry to the Reticle Hub.
    ///
    /// Use this when:
    /// - The MCP server uses HTTP/SSE/WebSocket transport
    /// - The MCP server is remote or started separately
    /// - You need to debug HTTP-based MCP traffic
    ///
    /// Example:
    ///   reticle proxy --name godaddy --listen 3001 --upstream http://localhost:8080
    ///
    /// Then configure Claude Desktop to connect to http://localhost:3001
    Proxy {
        /// Server name for identification in the dashboard
        #[arg(short, long, required = true)]
        name: String,

        /// Local port to listen on
        #[arg(short, long, default_value = "3001")]
        listen: u16,

        /// Upstream MCP server URL
        #[arg(short, long, required = true)]
        upstream: String,

        /// Socket path for Hub connection
        ///
        /// Defaults to /tmp/reticle.sock (or RETICLE_SOCKET env var)
        #[arg(long, env = "RETICLE_SOCKET")]
        socket: Option<String>,

        /// Disable telemetry (pure proxy mode)
        #[arg(long)]
        no_telemetry: bool,
    },
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum LogFormat {
    /// Human-readable text output
    Text,
    /// JSON output (one object per line)
    Json,
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();

    // Handle subcommands
    match cli.command {
        Some(Commands::Run {
            name,
            socket,
            no_telemetry,
            command,
        }) => run_wrapper(name, socket, no_telemetry, command).await,

        Some(Commands::Log {
            name,
            format,
            command,
        }) => run_logger(name, format, command).await,

        Some(Commands::Proxy {
            name,
            listen,
            upstream,
            socket,
            no_telemetry,
        }) => run_http_proxy_cmd(name, listen, upstream, socket, no_telemetry).await,

        None => {
            // Legacy mode: handle old-style invocation for backwards compatibility
            if !cli.command_args.is_empty() {
                // Old style: reticle [--gui] -- <command>
                let name = cli.name;
                let use_socket = cli.gui;
                run_legacy(name, use_socket, cli.command_args).await
            } else {
                eprintln!("Reticle - MCP Observability Proxy\n");
                eprintln!("Usage:");
                eprintln!("  reticle run --name <NAME> -- <COMMAND> [ARGS...]   # Wrap stdio MCP servers");
                eprintln!("  reticle proxy --name <NAME> -u <URL> -l <PORT>     # Proxy HTTP MCP servers");
                eprintln!("  reticle log --name <NAME> -- <COMMAND> [ARGS...]   # Debug to stderr");
                eprintln!();
                eprintln!("Examples:");
                eprintln!("  # Wrap a stdio MCP server for Claude Desktop:");
                eprintln!("  reticle run --name github -- npx -y @modelcontextprotocol/server-github");
                eprintln!();
                eprintln!("  # Proxy an HTTP-based MCP server:");
                eprintln!("  reticle proxy --name remote-server --upstream http://localhost:8080 --listen 3001");
                eprintln!();
                eprintln!("  # Quick debug output to stderr:");
                eprintln!("  reticle log --format json -- npx -y @anthropic/mcp-server-filesystem /tmp");
                eprintln!();
                eprintln!("For more information: reticle --help");
                ExitCode::SUCCESS
            }
        }
    }
}

/// Run in wrapper mode (connects to Hub via Unix socket)
async fn run_wrapper(
    name: Option<String>,
    socket: Option<String>,
    no_telemetry: bool,
    command: Vec<String>,
) -> ExitCode {
    if command.is_empty() {
        eprintln!("Error: No command specified");
        return ExitCode::FAILURE;
    }

    let cmd = &command[0];
    let args: Vec<&str> = command[1..].iter().map(|s| s.as_str()).collect();

    let server_name = name.unwrap_or_else(|| extract_server_name(cmd));

    if no_telemetry {
        // Pure proxy mode - no telemetry
        run_proxy_with_sink(cmd, &args, &server_name, NoOpEventSink, None).await
    } else {
        // Set socket path if provided
        if let Some(path) = socket {
            std::env::set_var("RETICLE_SOCKET", path);
        }

        // Connect to Hub (fail-open: continues even if Hub unavailable)
        // Returns (sink, inject_receiver) for bidirectional communication
        let (event_sink, inject_rx) = UnixSocketEventSink::new(server_name.clone()).await;
        run_proxy_with_sink(cmd, &args, &server_name, event_sink, Some(inject_rx)).await
    }
}

/// Run in logger mode (output to stderr)
async fn run_logger(name: Option<String>, format: LogFormat, command: Vec<String>) -> ExitCode {
    // Initialize tracing for log mode
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    if command.is_empty() {
        eprintln!("Error: No command specified");
        return ExitCode::FAILURE;
    }

    let cmd = &command[0];
    let args: Vec<&str> = command[1..].iter().map(|s| s.as_str()).collect();

    let server_name = name.unwrap_or_else(|| extract_server_name(cmd));
    let json_output = matches!(format, LogFormat::Json);
    let event_sink = StdoutEventSink::new(json_output);

    tracing::info!("Starting Reticle logger for '{}'", server_name);
    run_proxy_with_sink(cmd, &args, &server_name, event_sink, None).await
}

/// Legacy mode for backwards compatibility
async fn run_legacy(name: Option<String>, use_socket: bool, command: Vec<String>) -> ExitCode {
    if command.is_empty() {
        eprintln!("Error: No command specified");
        eprintln!("Usage: reticle run --name <NAME> -- <COMMAND> [ARGS...]");
        return ExitCode::FAILURE;
    }

    let cmd = &command[0];
    let args: Vec<&str> = command[1..].iter().map(|s| s.as_str()).collect();

    let server_name = name.unwrap_or_else(|| extract_server_name(cmd));

    if use_socket {
        // Old --gui flag behavior
        let (event_sink, inject_rx) = UnixSocketEventSink::new(server_name.clone()).await;
        run_proxy_with_sink(cmd, &args, &server_name, event_sink, Some(inject_rx)).await
    } else {
        // Pure passthrough (legacy default)
        run_proxy_with_sink(cmd, &args, &server_name, NoOpEventSink, None).await
    }
}

/// Run in HTTP proxy mode (intercepts HTTP-based MCP traffic)
async fn run_http_proxy_cmd(
    name: String,
    listen: u16,
    upstream: String,
    socket: Option<String>,
    no_telemetry: bool,
) -> ExitCode {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with_target(false)
        .with_writer(std::io::stderr)
        .init();

    if no_telemetry {
        // Pure proxy mode - no telemetry
        eprintln!("[HTTP PROXY] Running in pure proxy mode (no telemetry)");
        let event_sink = http_proxy::HttpEventSink::NoOp(NoOpEventSink);
        match http_proxy::run_http_proxy(upstream, listen, name, event_sink, None).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("[reticle] Error: {e}");
                ExitCode::FAILURE
            }
        }
    } else {
        // Set socket path if provided
        if let Some(path) = socket {
            std::env::set_var("RETICLE_SOCKET", path);
        }

        // Connect to Hub (fail-open: continues even if Hub unavailable)
        let (unix_sink, inject_rx) = UnixSocketEventSink::new(name.clone()).await;
        let event_sink = http_proxy::HttpEventSink::UnixSocket(std::sync::Arc::new(unix_sink));

        match http_proxy::run_http_proxy(upstream, listen, name, event_sink, Some(inject_rx)).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("[reticle] Error: {e}");
                ExitCode::FAILURE
            }
        }
    }
}

/// Extract server name from command path
fn extract_server_name(cmd: &str) -> String {
    std::path::Path::new(cmd)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "mcp-server".to_string())
}

/// Run the proxy with a given event sink
async fn run_proxy_with_sink<S: reticle_core::events::EventSink + 'static>(
    cmd: &str,
    args: &[&str],
    server_name: &str,
    event_sink: S,
    inject_rx: Option<InjectReceiver>,
) -> ExitCode {
    match proxy::run_stdio_proxy(cmd, args, server_name, event_sink, inject_rx).await {
        Ok(exit_code) => {
            if exit_code == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(exit_code as u8)
            }
        }
        Err(e) => {
            eprintln!("[reticle] Error: {e}");
            ExitCode::FAILURE
        }
    }
}
