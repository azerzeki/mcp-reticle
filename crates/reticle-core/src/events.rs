//! Event Sink Trait
//!
//! This module provides the EventSink trait for decoupling event emission
//! from GUI frameworks. Implementations can emit events to Tauri, write
//! to stdout (CLI), or any other sink.

use async_trait::async_trait;
use serde::Serialize;

use crate::protocol::LogEntry;
use crate::session_recorder::RecordedSession;

/// Event sink for emitting events to listeners
///
/// This trait abstracts event emission so proxy logic can work with
/// different frontends (Tauri GUI, CLI, tests, etc.)
#[async_trait]
pub trait EventSink: Send + Sync {
    /// Emit a log event (new message intercepted)
    async fn emit_log(&self, entry: &LogEntry) -> Result<(), String>;

    /// Emit a session started event
    async fn emit_session_started(
        &self,
        session_id: &str,
        session_name: &str,
    ) -> Result<(), String>;

    /// Emit a session ended event
    async fn emit_session_ended(&self, session_id: &str) -> Result<(), String>;

    /// Emit a recording started event
    async fn emit_recording_started(&self, session_id: &str) -> Result<(), String>;

    /// Emit a recording stopped event
    async fn emit_recording_stopped(&self, session: &RecordedSession) -> Result<(), String>;

    /// Emit a generic event with custom payload
    async fn emit_custom<T: Serialize + Send + Sync>(
        &self,
        event_name: &str,
        payload: &T,
    ) -> Result<(), String>;
}

/// No-op event sink for testing or CLI mode without event emission
#[derive(Default, Clone)]
pub struct NoOpEventSink;

#[async_trait]
impl EventSink for NoOpEventSink {
    async fn emit_log(&self, _entry: &LogEntry) -> Result<(), String> {
        Ok(())
    }

    async fn emit_session_started(
        &self,
        _session_id: &str,
        _session_name: &str,
    ) -> Result<(), String> {
        Ok(())
    }

    async fn emit_session_ended(&self, _session_id: &str) -> Result<(), String> {
        Ok(())
    }

    async fn emit_recording_started(&self, _session_id: &str) -> Result<(), String> {
        Ok(())
    }

    async fn emit_recording_stopped(&self, _session: &RecordedSession) -> Result<(), String> {
        Ok(())
    }

    async fn emit_custom<T: Serialize + Send + Sync>(
        &self,
        _event_name: &str,
        _payload: &T,
    ) -> Result<(), String> {
        Ok(())
    }
}

/// Stdout event sink for CLI mode - prints events to console
#[derive(Default, Clone)]
pub struct StdoutEventSink {
    /// Whether to print in JSON format
    pub json_output: bool,
}

impl StdoutEventSink {
    pub fn new(json_output: bool) -> Self {
        Self { json_output }
    }
}

#[async_trait]
impl EventSink for StdoutEventSink {
    async fn emit_log(&self, entry: &LogEntry) -> Result<(), String> {
        if self.json_output {
            println!("{}", serde_json::to_string(entry).unwrap_or_default());
        } else {
            let direction = match entry.direction {
                crate::protocol::Direction::In => "→",
                crate::protocol::Direction::Out => "←",
            };
            let method = entry.method.as_deref().unwrap_or("-");
            println!(
                "[{}] {} {}",
                format_timestamp(entry.timestamp),
                direction,
                method
            );
        }
        Ok(())
    }

    async fn emit_session_started(
        &self,
        session_id: &str,
        session_name: &str,
    ) -> Result<(), String> {
        if self.json_output {
            println!(
                r#"{{"event":"session_started","session_id":"{session_id}","name":"{session_name}"}}"#
            );
        } else {
            println!("Session started: {session_name} ({session_id})");
        }
        Ok(())
    }

    async fn emit_session_ended(&self, session_id: &str) -> Result<(), String> {
        if self.json_output {
            println!(r#"{{"event":"session_ended","session_id":"{session_id}"}}"#);
        } else {
            println!("Session ended: {session_id}");
        }
        Ok(())
    }

    async fn emit_recording_started(&self, session_id: &str) -> Result<(), String> {
        if self.json_output {
            println!(r#"{{"event":"recording_started","session_id":"{session_id}"}}"#);
        } else {
            println!("Recording started: {session_id}");
        }
        Ok(())
    }

    async fn emit_recording_stopped(&self, session: &RecordedSession) -> Result<(), String> {
        if self.json_output {
            println!(
                r#"{{"event":"recording_stopped","session_id":"{}","message_count":{}}}"#,
                session.id,
                session.messages.len()
            );
        } else {
            println!(
                "Recording stopped: {} ({} messages)",
                session.id,
                session.messages.len()
            );
        }
        Ok(())
    }

    async fn emit_custom<T: Serialize + Send + Sync>(
        &self,
        event_name: &str,
        payload: &T,
    ) -> Result<(), String> {
        if self.json_output {
            let payload_json = serde_json::to_string(payload).unwrap_or_default();
            println!(r#"{{"event":"{event_name}","payload":{payload_json}}}"#);
        } else {
            println!("[{event_name}] Custom event");
        }
        Ok(())
    }
}

fn format_timestamp(micros: u64) -> String {
    let millis = micros / 1000;
    let secs = millis / 1000;
    let mins = secs / 60;
    let hours = mins / 60;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        hours % 24,
        mins % 60,
        secs % 60,
        millis % 1000
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_sink() {
        let sink = NoOpEventSink;
        assert!(sink
            .emit_session_started("test", "Test Session")
            .await
            .is_ok());
        assert!(sink.emit_session_ended("test").await.is_ok());
    }

    #[test]
    fn test_format_timestamp() {
        assert_eq!(format_timestamp(0), "00:00:00.000");
        assert_eq!(format_timestamp(1_000_000), "00:00:01.000");
        assert_eq!(format_timestamp(3_661_500_000), "01:01:01.500");
    }
}
