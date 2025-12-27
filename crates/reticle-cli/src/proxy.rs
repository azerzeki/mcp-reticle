//! stdio proxy implementation for CLI
//!
//! This module implements a simple stdio proxy that wraps an MCP server
//! process and forwards all traffic while emitting events.

use reticle_core::events::EventSink;
use reticle_core::protocol::{Direction, LogEntry, MessageType};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

/// Run a stdio proxy for an MCP server
pub async fn run_stdio_proxy<E: EventSink>(
    command: &str,
    args: &[&str],
    server_name: &str,
    event_sink: E,
) -> Result<i32, String> {
    // Generate session ID
    let session_id = format!(
        "session-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
    );

    // Emit session started
    event_sink
        .emit_session_started(&session_id, server_name)
        .await
        .map_err(|e| format!("Failed to emit session started: {e}"))?;

    // Start the child process
    let mut child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start process: {e}"))?;

    let child_stdin = child.stdin.take().ok_or("Failed to get child stdin")?;
    let child_stdout = child.stdout.take().ok_or("Failed to get child stdout")?;
    let child_stderr = child.stderr.take().ok_or("Failed to get child stderr")?;

    // Set up readers
    let mut stdout_reader = BufReader::new(child_stdout).lines();
    let mut stderr_reader = BufReader::new(child_stderr).lines();
    let mut stdin_reader = BufReader::new(tokio::io::stdin()).lines();
    let mut child_stdin = child_stdin;

    let mut log_counter = 0u64;

    // Main proxy loop
    loop {
        tokio::select! {
            // Read from parent's stdin, write to child's stdin
            line = stdin_reader.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        log_counter += 1;
                        let log_id = format!("log-{log_counter}");

                        // Parse as JSON if possible
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                            let entry = LogEntry::with_server(
                                log_id,
                                session_id.clone(),
                                Direction::In,
                                json,
                                server_name.to_string(),
                            );
                            let _ = event_sink.emit_log(&entry).await;
                        } else {
                            let entry = LogEntry::new_raw_with_server(
                                log_id,
                                session_id.clone(),
                                Direction::In,
                                line.clone(),
                                MessageType::Raw,
                                server_name.to_string(),
                            );
                            let _ = event_sink.emit_log(&entry).await;
                        }

                        // Forward to child
                        if let Err(e) = child_stdin.write_all(line.as_bytes()).await {
                            tracing::error!("Failed to write to child stdin: {}", e);
                            break;
                        }
                        if let Err(e) = child_stdin.write_all(b"\n").await {
                            tracing::error!("Failed to write newline to child stdin: {}", e);
                            break;
                        }
                        let _ = child_stdin.flush().await;
                    }
                    Ok(None) => {
                        // Parent stdin closed
                        tracing::info!("Parent stdin closed");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("Error reading stdin: {}", e);
                        break;
                    }
                }
            }

            // Read from child's stdout, write to parent's stdout
            line = stdout_reader.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        log_counter += 1;
                        let log_id = format!("log-{log_counter}");

                        // Parse as JSON if possible
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&line) {
                            let entry = LogEntry::with_server(
                                log_id,
                                session_id.clone(),
                                Direction::Out,
                                json,
                                server_name.to_string(),
                            );
                            let _ = event_sink.emit_log(&entry).await;
                        } else {
                            let entry = LogEntry::new_raw_with_server(
                                log_id,
                                session_id.clone(),
                                Direction::Out,
                                line.clone(),
                                MessageType::Raw,
                                server_name.to_string(),
                            );
                            let _ = event_sink.emit_log(&entry).await;
                        }

                        // Forward to parent stdout
                        println!("{line}");
                    }
                    Ok(None) => {
                        // Child stdout closed
                        tracing::info!("Child stdout closed");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("Error reading child stdout: {}", e);
                        break;
                    }
                }
            }

            // Read from child's stderr, log it
            line = stderr_reader.next_line() => {
                match line {
                    Ok(Some(line)) => {
                        log_counter += 1;
                        let log_id = format!("log-{log_counter}");

                        let entry = LogEntry::new_raw_with_server(
                            log_id,
                            session_id.clone(),
                            Direction::Out,
                            line.clone(),
                            MessageType::Stderr,
                            server_name.to_string(),
                        );
                        let _ = event_sink.emit_log(&entry).await;

                        // Forward to parent stderr
                        eprintln!("{line}");
                    }
                    Ok(None) => {
                        // Child stderr closed - this is expected
                    }
                    Err(e) => {
                        tracing::error!("Error reading child stderr: {}", e);
                    }
                }
            }

            // Check if child has exited
            status = child.wait() => {
                match status {
                    Ok(status) => {
                        tracing::info!("Child process exited with: {}", status);
                        let _ = event_sink.emit_session_ended(&session_id).await;
                        return Ok(status.code().unwrap_or(0));
                    }
                    Err(e) => {
                        tracing::error!("Error waiting for child: {}", e);
                        let _ = event_sink.emit_session_ended(&session_id).await;
                        return Err(format!("Error waiting for child: {e}"));
                    }
                }
            }
        }
    }

    // Wait for child to exit
    let status = child
        .wait()
        .await
        .map_err(|e| format!("Error waiting for child: {e}"))?;
    let _ = event_sink.emit_session_ended(&session_id).await;

    Ok(status.code().unwrap_or(0))
}
