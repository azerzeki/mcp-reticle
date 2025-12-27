//! Session recording for MCP message capture and replay
//!
//! This module provides functionality to record complete MCP sessions,
//! including all messages exchanged between client and server, along
//! with timing information for accurate replay.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;

/// A complete recorded session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedSession {
    pub id: String,
    pub name: String,
    pub started_at: u64,
    pub ended_at: Option<u64>,
    pub messages: Vec<RecordedMessage>,
    pub metadata: SessionMetadata,
}

/// Individual recorded message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordedMessage {
    /// Unique message ID
    pub id: String,

    /// Absolute timestamp in microseconds since UNIX epoch
    pub timestamp_micros: u64,

    /// Time since session start in milliseconds
    pub relative_time_ms: u64,

    /// Message direction
    pub direction: MessageDirection,

    /// Message content (JSON-RPC)
    pub content: serde_json::Value,

    /// Additional metadata
    pub metadata: MessageMetadata,
}

/// Message direction
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageDirection {
    /// Message from client to server
    ToServer,
    /// Message from server to client
    ToClient,
}

impl std::fmt::Display for MessageDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageDirection::ToServer => write!(f, "to_server"),
            MessageDirection::ToClient => write!(f, "to_client"),
        }
    }
}

/// Metadata about a recorded message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    /// MCP method name (if applicable)
    pub method: Option<String>,

    /// JSON-RPC message ID (if applicable)
    pub jsonrpc_id: Option<serde_json::Value>,

    /// Whether this message was injected by the debugger
    pub injected: bool,

    /// Whether this message was modified by the debugger
    pub modified: bool,

    /// Size in bytes
    pub size_bytes: usize,
}

/// Session metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    /// Transport type used
    pub transport: String,

    /// Total message count
    pub message_count: usize,

    /// Session duration in milliseconds
    pub duration_ms: Option<u64>,

    /// Client information (if available)
    pub client_info: Option<ClientInfo>,

    /// Server information (if available)
    pub server_info: Option<ServerInfo>,

    /// Server identifier for multi-server support
    #[serde(default)]
    pub server_id: Option<ServerIdentifier>,

    /// Custom tags for filtering and organization
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Server identifier for multi-server tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerIdentifier {
    /// Human-readable server name (e.g., "claude-dev", "filesystem")
    pub name: String,

    /// Server version from MCP initialize response
    #[serde(default)]
    pub version: Option<String>,

    /// Command used to start the server
    pub command: String,

    /// Command arguments
    #[serde(default)]
    pub args: Vec<String>,

    /// Connection type: "stdio", "sse", "websocket"
    pub connection_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// Active session recorder
pub struct SessionRecorder {
    session_id: String,
    session_name: String,
    started_at: SystemTime,
    messages: Arc<Mutex<Vec<RecordedMessage>>>,
    transport_type: String,
    server_id: Option<ServerIdentifier>,
    tags: Arc<Mutex<Vec<String>>>,
}

impl SessionRecorder {
    /// Create a new session recorder
    pub fn new(session_id: String, session_name: String, transport_type: String) -> Self {
        Self {
            session_id,
            session_name,
            started_at: SystemTime::now(),
            messages: Arc::new(Mutex::new(Vec::new())),
            transport_type,
            server_id: None,
            tags: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a new session recorder with server identifier
    pub fn with_server(
        session_id: String,
        session_name: String,
        transport_type: String,
        server_id: ServerIdentifier,
    ) -> Self {
        Self {
            session_id,
            session_name,
            started_at: SystemTime::now(),
            messages: Arc::new(Mutex::new(Vec::new())),
            transport_type,
            server_id: Some(server_id),
            tags: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a tag to the session
    pub async fn add_tag(&self, tag: String) {
        let mut tags = self.tags.lock().await;
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }

    /// Remove a tag from the session
    pub async fn remove_tag(&self, tag: &str) {
        let mut tags = self.tags.lock().await;
        tags.retain(|t| t != tag);
    }

    /// Get all tags
    pub async fn get_tags(&self) -> Vec<String> {
        self.tags.lock().await.clone()
    }

    /// Get server identifier
    pub fn get_server_id(&self) -> Option<&ServerIdentifier> {
        self.server_id.as_ref()
    }

    /// Record a message
    pub async fn record_message(
        &self,
        content: serde_json::Value,
        direction: MessageDirection,
    ) -> Result<(), RecorderError> {
        let now = SystemTime::now();
        let timestamp_micros = now
            .duration_since(UNIX_EPOCH)
            .map_err(|e| RecorderError::TimeError(e.to_string()))?
            .as_micros() as u64;

        let relative_time_ms = now
            .duration_since(self.started_at)
            .map_err(|e| RecorderError::TimeError(e.to_string()))?
            .as_millis() as u64;

        // Extract metadata from message
        let method = content
            .get("method")
            .and_then(|v| v.as_str())
            .map(String::from);

        let jsonrpc_id = content.get("id").cloned();

        let content_str = serde_json::to_string(&content)
            .map_err(|e| RecorderError::SerializationError(e.to_string()))?;
        let size_bytes = content_str.len();

        let message = RecordedMessage {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp_micros,
            relative_time_ms,
            direction,
            content,
            metadata: MessageMetadata {
                method,
                jsonrpc_id,
                injected: false,
                modified: false,
                size_bytes,
            },
        };

        let mut messages = self.messages.lock().await;
        messages.push(message);

        Ok(())
    }

    /// Finalize the recording and return the complete session
    pub async fn finalize(self) -> Result<RecordedSession, RecorderError> {
        let ended_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| RecorderError::TimeError(e.to_string()))?
            .as_micros() as u64;

        let started_at = self
            .started_at
            .duration_since(UNIX_EPOCH)
            .map_err(|e| RecorderError::TimeError(e.to_string()))?
            .as_micros() as u64;

        let messages = self.messages.lock().await.clone();
        let message_count = messages.len();
        let tags = self.tags.lock().await.clone();

        let duration_ms = (ended_at - started_at) / 1000;

        Ok(RecordedSession {
            id: self.session_id,
            name: self.session_name,
            started_at,
            ended_at: Some(ended_at),
            messages,
            metadata: SessionMetadata {
                transport: self.transport_type,
                message_count,
                duration_ms: Some(duration_ms),
                client_info: None, // Will be populated from initialize message
                server_info: None, // Will be populated from initialize response
                server_id: self.server_id,
                tags,
            },
        })
    }

    /// Get current session statistics
    pub async fn get_stats(&self) -> RecorderStats {
        let messages = self.messages.lock().await;
        let message_count = messages.len();

        let to_server = messages
            .iter()
            .filter(|m| m.direction == MessageDirection::ToServer)
            .count();

        let to_client = messages
            .iter()
            .filter(|m| m.direction == MessageDirection::ToClient)
            .count();

        let elapsed = SystemTime::now()
            .duration_since(self.started_at)
            .unwrap_or_default();

        RecorderStats {
            session_id: self.session_id.clone(),
            message_count,
            to_server_count: to_server,
            to_client_count: to_client,
            duration_seconds: elapsed.as_secs(),
        }
    }
}

/// Recording statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecorderStats {
    pub session_id: String,
    pub message_count: usize,
    pub to_server_count: usize,
    pub to_client_count: usize,
    pub duration_seconds: u64,
}

/// Recorder errors
#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
#[allow(clippy::enum_variant_names)]
pub enum RecorderError {
    #[error("Time error: {0}")]
    TimeError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Storage error: {0}")]
    StorageError(String),
}

// UUID support - add to Cargo.toml dependencies later
mod uuid {
    use std::fmt;

    pub struct Uuid([u8; 16]);

    impl Uuid {
        pub fn new_v4() -> Self {
            let mut bytes = [0u8; 16];
            // Simple random UUID - in production use uuid crate
            for b in &mut bytes {
                *b = (rand() * 256.0) as u8;
            }
            bytes[6] = (bytes[6] & 0x0f) | 0x40; // Version 4
            bytes[8] = (bytes[8] & 0x3f) | 0x80; // Variant
            Uuid(bytes)
        }
    }

    impl fmt::Display for Uuid {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(
                f,
                "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
                self.0[0], self.0[1], self.0[2], self.0[3],
                self.0[4], self.0[5],
                self.0[6], self.0[7],
                self.0[8], self.0[9],
                self.0[10], self.0[11], self.0[12], self.0[13], self.0[14], self.0[15]
            )
        }
    }

    fn rand() -> f64 {
        use std::time::SystemTime;
        let now = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        ((now.as_nanos() % 1000000) as f64) / 1000000.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_direction_display() {
        assert_eq!(MessageDirection::ToServer.to_string(), "to_server");
        assert_eq!(MessageDirection::ToClient.to_string(), "to_client");
    }

    #[test]
    fn test_message_direction_serialize() {
        let to_server = serde_json::to_string(&MessageDirection::ToServer).unwrap();
        let to_client = serde_json::to_string(&MessageDirection::ToClient).unwrap();

        assert_eq!(to_server, "\"toserver\"");
        assert_eq!(to_client, "\"toclient\"");
    }

    #[test]
    fn test_message_direction_deserialize() {
        let to_server: MessageDirection = serde_json::from_str("\"toserver\"").unwrap();
        let to_client: MessageDirection = serde_json::from_str("\"toclient\"").unwrap();

        assert_eq!(to_server, MessageDirection::ToServer);
        assert_eq!(to_client, MessageDirection::ToClient);
    }

    #[test]
    fn test_session_recorder_new() {
        let recorder = SessionRecorder::new(
            "session-1".to_string(),
            "Test Session".to_string(),
            "stdio".to_string(),
        );

        assert!(recorder.get_server_id().is_none());
    }

    #[test]
    fn test_session_recorder_with_server() {
        let server_id = ServerIdentifier {
            name: "filesystem-server".to_string(),
            version: Some("1.0.0".to_string()),
            command: "npx".to_string(),
            args: vec!["@modelcontextprotocol/server-filesystem".to_string()],
            connection_type: "stdio".to_string(),
        };

        let recorder = SessionRecorder::with_server(
            "session-2".to_string(),
            "Test Session".to_string(),
            "stdio".to_string(),
            server_id,
        );

        let server = recorder.get_server_id().unwrap();
        assert_eq!(server.name, "filesystem-server");
        assert_eq!(server.version, Some("1.0.0".to_string()));
    }

    #[tokio::test]
    async fn test_session_recorder_tags() {
        let recorder = SessionRecorder::new(
            "session-3".to_string(),
            "Tag Test".to_string(),
            "stdio".to_string(),
        );

        // Add tags
        recorder.add_tag("production".to_string()).await;
        recorder.add_tag("debugging".to_string()).await;

        let tags = recorder.get_tags().await;
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&"production".to_string()));
        assert!(tags.contains(&"debugging".to_string()));

        // Add duplicate tag (should not increase count)
        recorder.add_tag("production".to_string()).await;
        let tags = recorder.get_tags().await;
        assert_eq!(tags.len(), 2);

        // Remove tag
        recorder.remove_tag("debugging").await;
        let tags = recorder.get_tags().await;
        assert_eq!(tags.len(), 1);
        assert!(tags.contains(&"production".to_string()));
    }

    #[tokio::test]
    async fn test_session_recorder_record_message() {
        let recorder = SessionRecorder::new(
            "session-4".to_string(),
            "Message Test".to_string(),
            "stdio".to_string(),
        );

        let content = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/list",
            "id": 1
        });

        recorder.record_message(content, MessageDirection::ToServer).await.unwrap();

        let stats = recorder.get_stats().await;
        assert_eq!(stats.message_count, 1);
        assert_eq!(stats.to_server_count, 1);
        assert_eq!(stats.to_client_count, 0);
    }

    #[tokio::test]
    async fn test_session_recorder_get_stats() {
        let recorder = SessionRecorder::new(
            "session-5".to_string(),
            "Stats Test".to_string(),
            "stdio".to_string(),
        );

        // Add multiple messages
        let request = serde_json::json!({"jsonrpc": "2.0", "method": "ping", "id": 1});
        let response = serde_json::json!({"jsonrpc": "2.0", "result": {}, "id": 1});

        recorder.record_message(request, MessageDirection::ToServer).await.unwrap();
        recorder.record_message(response, MessageDirection::ToClient).await.unwrap();

        let stats = recorder.get_stats().await;
        assert_eq!(stats.session_id, "session-5");
        assert_eq!(stats.message_count, 2);
        assert_eq!(stats.to_server_count, 1);
        assert_eq!(stats.to_client_count, 1);
    }

    #[tokio::test]
    async fn test_session_recorder_finalize() {
        let recorder = SessionRecorder::new(
            "session-6".to_string(),
            "Finalize Test".to_string(),
            "sse".to_string(),
        );

        recorder.add_tag("test".to_string()).await;

        let content = serde_json::json!({"jsonrpc": "2.0", "method": "initialize", "id": 1});
        recorder.record_message(content, MessageDirection::ToServer).await.unwrap();

        let session = recorder.finalize().await.unwrap();

        assert_eq!(session.id, "session-6");
        assert_eq!(session.name, "Finalize Test");
        assert!(session.ended_at.is_some());
        assert_eq!(session.messages.len(), 1);
        assert_eq!(session.metadata.transport, "sse");
        assert_eq!(session.metadata.message_count, 1);
        assert!(session.metadata.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_recorded_message_metadata_extraction() {
        let metadata = MessageMetadata {
            method: Some("tools/call".to_string()),
            jsonrpc_id: Some(serde_json::json!(42)),
            injected: false,
            modified: true,
            size_bytes: 256,
        };

        assert_eq!(metadata.method, Some("tools/call".to_string()));
        assert_eq!(metadata.jsonrpc_id, Some(serde_json::json!(42)));
        assert!(!metadata.injected);
        assert!(metadata.modified);
        assert_eq!(metadata.size_bytes, 256);
    }

    #[test]
    fn test_server_identifier_serialization() {
        let server_id = ServerIdentifier {
            name: "test-server".to_string(),
            version: Some("2.0.0".to_string()),
            command: "node".to_string(),
            args: vec!["server.js".to_string(), "--port".to_string(), "3000".to_string()],
            connection_type: "websocket".to_string(),
        };

        let json = serde_json::to_string(&server_id).unwrap();
        assert!(json.contains("\"name\":\"test-server\""));
        assert!(json.contains("\"version\":\"2.0.0\""));
        assert!(json.contains("\"connection_type\":\"websocket\""));

        let deserialized: ServerIdentifier = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.name, "test-server");
        assert_eq!(deserialized.args.len(), 3);
    }

    #[test]
    fn test_client_server_info() {
        let client = ClientInfo {
            name: "Claude Desktop".to_string(),
            version: "1.0.0".to_string(),
        };

        let server = ServerInfo {
            name: "MCP Server".to_string(),
            version: "0.1.0".to_string(),
        };

        let client_json = serde_json::to_string(&client).unwrap();
        let server_json = serde_json::to_string(&server).unwrap();

        assert!(client_json.contains("Claude Desktop"));
        assert!(server_json.contains("MCP Server"));
    }

    #[test]
    fn test_session_metadata_defaults() {
        let metadata = SessionMetadata {
            transport: "stdio".to_string(),
            message_count: 0,
            duration_ms: None,
            client_info: None,
            server_info: None,
            server_id: None,
            tags: vec![],
        };

        assert!(metadata.client_info.is_none());
        assert!(metadata.server_info.is_none());
        assert!(metadata.server_id.is_none());
        assert!(metadata.tags.is_empty());
    }

    #[test]
    fn test_recorder_error_display() {
        let time_err = RecorderError::TimeError("time went backwards".to_string());
        let ser_err = RecorderError::SerializationError("invalid json".to_string());
        let storage_err = RecorderError::StorageError("disk full".to_string());

        assert_eq!(time_err.to_string(), "Time error: time went backwards");
        assert_eq!(ser_err.to_string(), "Serialization error: invalid json");
        assert_eq!(storage_err.to_string(), "Storage error: disk full");
    }

    #[test]
    fn test_uuid_format() {
        let uuid = uuid::Uuid::new_v4();
        let uuid_str = uuid.to_string();

        // UUID v4 format: xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx
        assert_eq!(uuid_str.len(), 36);
        assert_eq!(&uuid_str[8..9], "-");
        assert_eq!(&uuid_str[13..14], "-");
        assert_eq!(&uuid_str[14..15], "4"); // Version 4
        assert_eq!(&uuid_str[18..19], "-");
        assert_eq!(&uuid_str[23..24], "-");
    }

    #[test]
    fn test_recorded_session_serialization() {
        let session = RecordedSession {
            id: "session-test".to_string(),
            name: "Test".to_string(),
            started_at: 1700000000000000,
            ended_at: Some(1700000001000000),
            messages: vec![],
            metadata: SessionMetadata {
                transport: "stdio".to_string(),
                message_count: 0,
                duration_ms: Some(1000),
                client_info: None,
                server_info: None,
                server_id: None,
                tags: vec!["test".to_string()],
            },
        };

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: RecordedSession = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.id, "session-test");
        assert_eq!(deserialized.metadata.duration_ms, Some(1000));
        assert!(deserialized.metadata.tags.contains(&"test".to_string()));
    }
}
