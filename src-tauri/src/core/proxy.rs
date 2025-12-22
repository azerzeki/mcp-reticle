use bytes::BytesMut;
use std::io;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncReadExt;
use tokio::process::Child;
use tokio::sync::Mutex;
use tracing::{debug, error, trace, warn};

use super::protocol::{Direction, LogEntry, MessageType};
use super::session_recorder::{MessageDirection, SessionRecorder};

/// Global message counter for generating unique IDs
static MESSAGE_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Main proxy loop for Tauri desktop app
///
/// This is the heart of Reticle. It monitors the MCP server child process
/// and intercepts JSON-RPC messages from its stdout for real-time visualization.
///
/// Key design decisions:
/// - Desktop app mode: only monitors child stdout, no stdin forwarding needed
/// - Using BytesMut for zero-copy buffering where possible
/// - Line-based framing with \n delimiter per JSON-RPC spec
/// - Non-blocking partial line handling to maintain low latency
/// - Graceful EOF/error handling without panics
/// - Emits events to Tauri frontend for real-time updates
pub async fn run_proxy(
    mut child: Child,
    session_id: String,
    app_handle: AppHandle,
    recorder: Arc<Mutex<Option<SessionRecorder>>>,
) -> Result<(), io::Error> {
    // Get child's stdio handles (only stdout and stderr for monitoring)
    let mut child_stdout = child
        .stdout
        .take()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to open child stdout"))?;

    let mut child_stderr = child
        .stderr
        .take()
        .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to open child stderr"))?;

    // For Tauri desktop app, we don't have host stdin/stdout
    // We just monitor the child's stdout/stderr and emit events

    // Buffer for accumulating partial lines from stdout
    let mut stdout_buf = BytesMut::with_capacity(8192);

    // Read buffers
    let mut child_stdout_read_buf = vec![0u8; 4096];
    let mut child_stderr_read_buf = vec![0u8; 4096];

    eprintln!("[PROXY] Proxy loop started for session {session_id}");
    debug!("Proxy loop started for session {}", session_id);

    // Main event loop - monitor child stdout/stderr only
    loop {
        tokio::select! {
            biased;

            // Child stdout - parse JSON-RPC and emit events
            result = child_stdout.read(&mut child_stdout_read_buf) => {
                match result {
                    Ok(0) => {
                        eprintln!("[PROXY] Child stdout closed (EOF) - child process finished");
                        eprintln!("[PROXY] Buffer had {} bytes remaining", stdout_buf.len());
                        debug!("Child stdout closed (EOF)");
                        // Child process finished
                        return Ok(());
                    }
                    Ok(n) => {
                        eprintln!("[PROXY DEBUG] Read {n} bytes from child stdout");
                        trace!("Read {} bytes from child stdout", n);

                        // Append to buffer
                        stdout_buf.extend_from_slice(&child_stdout_read_buf[..n]);

                        // Process complete lines
                        while let Some(line_end) = find_newline(&stdout_buf) {
                            let line_with_newline = stdout_buf.split_to(line_end + 1);

                            // Parse and log (without newline)
                            let line_bytes = &line_with_newline[..line_end];
                            if let Ok(line_str) = std::str::from_utf8(line_bytes) {
                                if !line_str.trim().is_empty() {
                                    eprintln!("[PROXY DEBUG] Processing line: {}", &line_str[..line_str.len().min(100)]);
                                    let id = generate_message_id();

                                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(line_str) {
                                        // Valid JSON-RPC message
                                        eprintln!("[PROXY DEBUG] Parsed JSON, emitting log-event");
                                        debug!("Out: {}", line_str);

                                        let entry = LogEntry::new(
                                            id,
                                            session_id.clone(),
                                            Direction::Out,
                                            json.clone(),
                                        );

                                        // Emit to frontend
                                        if let Err(e) = app_handle.emit("log-event", &entry) {
                                            eprintln!("[PROXY ERROR] Failed to emit: {e}");
                                            warn!("Failed to emit log event: {}", e);
                                        } else {
                                            eprintln!("[PROXY DEBUG] Successfully emitted log-event");
                                        }

                                        // Record message if recording is active
                                        let recorder_lock = recorder.lock().await;
                                        if let Some(ref rec) = *recorder_lock {
                                            if let Err(e) = rec.record_message(json, MessageDirection::ToClient).await {
                                                warn!("Failed to record message: {}", e);
                                            }
                                        }
                                        drop(recorder_lock);
                                    } else {
                                        // Non-JSON output (error messages, tracebacks, etc.)
                                        eprintln!("[PROXY DEBUG] Non-JSON stdout, emitting as raw: {}", &line_str[..line_str.len().min(100)]);
                                        debug!("Raw stdout: {}", line_str);

                                        let entry = LogEntry::new_raw(
                                            id,
                                            session_id.clone(),
                                            Direction::Out,
                                            line_str.to_string(),
                                            MessageType::Raw,
                                        );

                                        // Emit to frontend
                                        if let Err(e) = app_handle.emit("log-event", &entry) {
                                            eprintln!("[PROXY ERROR] Failed to emit raw: {e}");
                                            warn!("Failed to emit raw log event: {}", e);
                                        } else {
                                            eprintln!("[PROXY DEBUG] Successfully emitted raw log-event");
                                        }
                                    }
                                }
                            }
                        }

                        // Prevent buffer bloat
                        if stdout_buf.len() > 65536 {
                            warn!("Large stdout buffer without newline, clearing");
                            stdout_buf.clear();
                        }
                    }
                    Err(e) => {
                        if e.kind() != io::ErrorKind::Interrupted {
                            error!("Error reading from child stdout: {}", e);
                            return Err(e);
                        }
                    }
                }
            }

            // Child stderr - emit as stderr message type for debugging visibility
            result = child_stderr.read(&mut child_stderr_read_buf) => {
                match result {
                    Ok(0) => {
                        debug!("Child stderr closed");
                    }
                    Ok(n) => {
                        // Emit stderr output to frontend for debugging
                        if let Ok(stderr_str) = std::str::from_utf8(&child_stderr_read_buf[..n]) {
                            let stderr_trimmed = stderr_str.trim();
                            if !stderr_trimmed.is_empty() {
                                eprintln!("[PROXY DEBUG] Stderr: {}", &stderr_trimmed[..stderr_trimmed.len().min(200)]);
                                debug!("Child stderr: {}", stderr_trimmed);

                                let id = generate_message_id();
                                let entry = LogEntry::new_raw(
                                    id,
                                    session_id.clone(),
                                    Direction::Out,
                                    stderr_trimmed.to_string(),
                                    MessageType::Stderr,
                                );

                                // Emit to frontend so user sees error output
                                if let Err(e) = app_handle.emit("log-event", &entry) {
                                    eprintln!("[PROXY ERROR] Failed to emit stderr: {e}");
                                    warn!("Failed to emit stderr log event: {}", e);
                                } else {
                                    eprintln!("[PROXY DEBUG] Successfully emitted stderr log-event");
                                }
                            }
                        }
                    }
                    Err(e) => {
                        if e.kind() != io::ErrorKind::Interrupted {
                            warn!("Error reading from child stderr: {}", e);
                        }
                    }
                }
            }
        }
    }
}

/// Find the position of the first newline in the buffer
#[inline]
fn find_newline(buf: &BytesMut) -> Option<usize> {
    buf.iter().position(|&b| b == b'\n')
}

/// Generate a unique message ID
fn generate_message_id() -> String {
    let counter = MESSAGE_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("msg-{counter}")
}
