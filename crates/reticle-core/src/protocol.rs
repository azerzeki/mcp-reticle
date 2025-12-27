//! JSON-RPC Protocol Types
//!
//! Core types for MCP (Model Context Protocol) JSON-RPC messages.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::token_counter::TokenCounter;

/// Direction of message flow through the proxy
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// From host (client) to child (server) - incoming
    #[serde(rename = "in")]
    In,
    /// From child (server) to host (client) - outgoing
    #[serde(rename = "out")]
    Out,
}

/// Type of message content
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum MessageType {
    /// Valid JSON-RPC message
    #[default]
    JsonRpc,
    /// Raw text output (non-JSON from stdout)
    Raw,
    /// Error output from stderr
    Stderr,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Direction::In => write!(f, "in"),
            Direction::Out => write!(f, "out"),
        }
    }
}

/// A logged JSON-RPC message with metadata
/// This matches the frontend LogEntry type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unique ID for this log entry
    pub id: String,
    /// Session ID this log belongs to
    pub session_id: String,
    /// When the message was intercepted (microseconds since UNIX_EPOCH)
    pub timestamp: u64,
    /// Direction of the message
    pub direction: Direction,
    /// The JSON-RPC message content as string
    pub content: String,
    /// Optional: extracted method from JSON-RPC
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    /// Optional: processing duration in microseconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_micros: Option<u64>,
    /// Type of message content (jsonrpc, raw, stderr)
    #[serde(default)]
    pub message_type: MessageType,
    /// Estimated token count for this message
    #[serde(default)]
    pub token_count: u64,
    /// Server name for multi-server filtering
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub server_name: Option<String>,
}

impl LogEntry {
    /// Create a new log entry from a JSON-RPC message
    pub fn new(
        id: String,
        session_id: String,
        direction: Direction,
        content: serde_json::Value,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        let method = extract_method(&content);
        let content_str = serde_json::to_string(&content).unwrap_or_default();

        // Count LLM-relevant tokens (extracts payload, not JSON-RPC overhead)
        let token_count = TokenCounter::count_mcp_context_tokens(&content);

        Self {
            id,
            session_id,
            timestamp,
            direction,
            content: content_str,
            method,
            duration_micros: None,
            message_type: MessageType::JsonRpc,
            token_count,
            server_name: None,
        }
    }

    /// Create a new log entry from a JSON-RPC message with server name
    pub fn with_server(
        id: String,
        session_id: String,
        direction: Direction,
        content: serde_json::Value,
        server_name: String,
    ) -> Self {
        let mut entry = Self::new(id, session_id, direction, content);
        entry.server_name = Some(server_name);
        entry
    }

    /// Create a new log entry from raw text (non-JSON output)
    pub fn new_raw(
        id: String,
        session_id: String,
        direction: Direction,
        content: String,
        message_type: MessageType,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_micros() as u64;

        // Estimate token count for raw content
        let token_count = TokenCounter::estimate_tokens(&content);

        Self {
            id,
            session_id,
            timestamp,
            direction,
            content,
            method: None,
            duration_micros: None,
            message_type,
            token_count,
            server_name: None,
        }
    }

    /// Create a new log entry from raw text with server name
    pub fn new_raw_with_server(
        id: String,
        session_id: String,
        direction: Direction,
        content: String,
        message_type: MessageType,
        server_name: String,
    ) -> Self {
        let mut entry = Self::new_raw(id, session_id, direction, content, message_type);
        entry.server_name = Some(server_name);
        entry
    }
}

/// Extract the method field from a JSON-RPC message if present
fn extract_method(value: &serde_json::Value) -> Option<String> {
    value
        .get("method")
        .and_then(|m| m.as_str())
        .map(|s| s.to_string())
}

/// JSON-RPC 2.0 Request structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<serde_json::Value>,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 Response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

/// JSON-RPC 2.0 Error structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 Notification structure (no id field)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direction_display() {
        assert_eq!(Direction::In.to_string(), "in");
        assert_eq!(Direction::Out.to_string(), "out");
    }

    #[test]
    fn test_direction_serde() {
        let dir_in = Direction::In;
        let json = serde_json::to_string(&dir_in).unwrap();
        assert_eq!(json, "\"in\"");

        let parsed: Direction = serde_json::from_str("\"out\"").unwrap();
        assert_eq!(parsed, Direction::Out);
    }

    #[test]
    fn test_message_type_default() {
        let mt: MessageType = Default::default();
        assert_eq!(mt, MessageType::JsonRpc);
    }

    #[test]
    fn test_log_entry_new() {
        let content = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "tools/call",
            "params": { "name": "test" },
            "id": 1
        });

        let entry = LogEntry::new(
            "log-1".to_string(),
            "session-1".to_string(),
            Direction::In,
            content,
        );

        assert_eq!(entry.id, "log-1");
        assert_eq!(entry.session_id, "session-1");
        assert_eq!(entry.direction, Direction::In);
        assert_eq!(entry.method, Some("tools/call".to_string()));
        assert_eq!(entry.message_type, MessageType::JsonRpc);
        assert!(entry.token_count > 0);
    }

    #[test]
    fn test_log_entry_with_server() {
        let content = serde_json::json!({"method": "test"});
        let entry = LogEntry::with_server(
            "log-1".to_string(),
            "session-1".to_string(),
            Direction::Out,
            content,
            "my-server".to_string(),
        );

        assert_eq!(entry.server_name, Some("my-server".to_string()));
    }

    #[test]
    fn test_log_entry_raw() {
        let entry = LogEntry::new_raw(
            "log-1".to_string(),
            "session-1".to_string(),
            Direction::Out,
            "Hello world".to_string(),
            MessageType::Raw,
        );

        assert_eq!(entry.content, "Hello world");
        assert_eq!(entry.message_type, MessageType::Raw);
        assert!(entry.method.is_none());
    }

    #[test]
    fn test_extract_method() {
        let with_method = serde_json::json!({"method": "test/method"});
        assert_eq!(
            extract_method(&with_method),
            Some("test/method".to_string())
        );

        let without_method = serde_json::json!({"result": {}});
        assert_eq!(extract_method(&without_method), None);
    }

    #[test]
    fn test_jsonrpc_request_serde() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "test".to_string(),
            params: Some(serde_json::json!({"key": "value"})),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"jsonrpc\":\"2.0\""));
        assert!(json.contains("\"method\":\"test\""));

        let parsed: JsonRpcRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.method, "test");
    }

    #[test]
    fn test_jsonrpc_response_with_result() {
        let resp = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            result: Some(serde_json::json!({"data": "value"})),
            error: None,
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"result\""));
        assert!(!json.contains("\"error\""));
    }

    #[test]
    fn test_jsonrpc_response_with_error() {
        let resp = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: serde_json::json!(1),
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid Request".to_string(),
                data: None,
            }),
        };

        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"error\""));
        assert!(json.contains("-32600"));
    }
}
