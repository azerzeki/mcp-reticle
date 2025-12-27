//! Reticle CLI
//!
//! Command-line interface for the Reticle MCP debugging proxy.
//! This binary wraps MCP server processes and forwards all traffic
//! while logging messages for debugging.

use clap::Parser;
use reticle_core::events::StdoutEventSink;
use std::process::ExitCode;
use tracing_subscriber::EnvFilter;

mod proxy;

/// Reticle - The Wireshark for the Model Context Protocol
///
/// Wrap your MCP server command to intercept and log all JSON-RPC traffic.
#[derive(Parser, Debug)]
#[command(name = "reticle")]
#[command(version, about, long_about = None)]
struct Args {
    /// Port for the Reticle dashboard to connect (optional for CLI mode)
    #[arg(short, long, default_value = "3001")]
    port: u16,

    /// Output format: text or json
    #[arg(short, long, default_value = "text")]
    format: OutputFormat,

    /// Server name for identification
    #[arg(short, long)]
    name: Option<String>,

    /// The command and arguments to run
    #[arg(trailing_var_arg = true, required = true)]
    command: Vec<String>,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[tokio::main]
async fn main() -> ExitCode {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .with_target(false)
        .init();

    let args = Args::parse();

    if args.command.is_empty() {
        eprintln!("Error: No command specified");
        eprintln!("Usage: reticle [OPTIONS] -- <command> [args...]");
        return ExitCode::FAILURE;
    }

    let command = &args.command[0];
    let cmd_args: Vec<&str> = args.command[1..].iter().map(|s| s.as_str()).collect();

    let server_name = args.name.unwrap_or_else(|| {
        // Extract server name from command
        std::path::Path::new(command)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "mcp-server".to_string())
    });

    let json_output = matches!(args.format, OutputFormat::Json);
    let event_sink = StdoutEventSink::new(json_output);

    tracing::info!(
        "Starting Reticle proxy for '{}' on port {}",
        server_name,
        args.port
    );

    match proxy::run_stdio_proxy(command, &cmd_args, &server_name, event_sink).await {
        Ok(exit_code) => {
            if exit_code == 0 {
                ExitCode::SUCCESS
            } else {
                ExitCode::from(exit_code as u8)
            }
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}
