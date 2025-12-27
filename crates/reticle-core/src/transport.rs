//! Transport types for MCP communication

use serde::{Deserialize, Serialize};
use std::fmt;

/// Transport type for MCP communication
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    /// stdio-based transport (stdin/stdout pipes)
    Stdio,
    /// HTTP-based transport with Server-Sent Events (legacy, protocol version 2024-11-05)
    Http,
    /// Streamable HTTP transport (protocol version 2025-03-26)
    /// Bidirectional HTTP with optional SSE streaming
    Streamable,
    /// WebSocket transport for real-time bidirectional communication
    WebSocket,
}

impl fmt::Display for TransportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportType::Stdio => write!(f, "stdio"),
            TransportType::Http => write!(f, "http"),
            TransportType::Streamable => write!(f, "streamable"),
            TransportType::WebSocket => write!(f, "websocket"),
        }
    }
}

/// Configuration for starting a transport
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TransportConfig {
    /// stdio transport configuration
    Stdio { command: String, args: Vec<String> },
    /// HTTP/SSE transport configuration (legacy)
    Http { server_url: String, proxy_port: u16 },
    /// Streamable HTTP transport configuration (MCP 2025-03-26)
    Streamable { server_url: String, proxy_port: u16 },
    /// WebSocket transport configuration
    WebSocket { server_url: String, proxy_port: u16 },
}

impl TransportConfig {
    /// Get the transport type from config
    #[allow(dead_code)]
    pub fn transport_type(&self) -> TransportType {
        match self {
            TransportConfig::Stdio { .. } => TransportType::Stdio,
            TransportConfig::Http { .. } => TransportType::Http,
            TransportConfig::Streamable { .. } => TransportType::Streamable,
            TransportConfig::WebSocket { .. } => TransportType::WebSocket,
        }
    }

    /// Check if this is demo mode (empty command or "demo")
    pub fn is_demo(&self) -> bool {
        match self {
            TransportConfig::Stdio { command, .. } => command.is_empty() || command == "demo",
            TransportConfig::Http { .. } => false,
            TransportConfig::Streamable { .. } => false,
            TransportConfig::WebSocket { .. } => false,
        }
    }
}

/// Errors that can occur with transports
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("Transport already running")]
    AlreadyRunning,

    #[error("Transport not running")]
    NotRunning,

    #[error("Failed to start transport: {0}")]
    StartFailed(String),

    #[error("Failed to stop transport: {0}")]
    StopFailed(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(String),

    #[error("WebSocket error: {0}")]
    WebSocket(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

impl From<TransportError> for String {
    fn from(err: TransportError) -> String {
        err.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_type_display() {
        assert_eq!(TransportType::Stdio.to_string(), "stdio");
        assert_eq!(TransportType::Http.to_string(), "http");
        assert_eq!(TransportType::Streamable.to_string(), "streamable");
        assert_eq!(TransportType::WebSocket.to_string(), "websocket");
    }

    #[test]
    fn test_transport_config_type() {
        let stdio = TransportConfig::Stdio {
            command: "test".to_string(),
            args: vec![],
        };
        assert_eq!(stdio.transport_type(), TransportType::Stdio);

        let http = TransportConfig::Http {
            server_url: "http://localhost".to_string(),
            proxy_port: 3000,
        };
        assert_eq!(http.transport_type(), TransportType::Http);
    }

    #[test]
    fn test_is_demo() {
        let demo1 = TransportConfig::Stdio {
            command: "".to_string(),
            args: vec![],
        };
        assert!(demo1.is_demo());

        let demo2 = TransportConfig::Stdio {
            command: "demo".to_string(),
            args: vec![],
        };
        assert!(demo2.is_demo());

        let real = TransportConfig::Stdio {
            command: "npx".to_string(),
            args: vec!["-y".to_string()],
        };
        assert!(!real.is_demo());
    }

    #[test]
    fn test_transport_config_serialization() {
        let config = TransportConfig::Stdio {
            command: "npx".to_string(),
            args: vec!["-y".to_string(), "mcp-server".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("stdio"));
        assert!(json.contains("npx"));

        let parsed: TransportConfig = serde_json::from_str(&json).unwrap();
        if let TransportConfig::Stdio { command, args } = parsed {
            assert_eq!(command, "npx");
            assert_eq!(args, vec!["-y", "mcp-server"]);
        } else {
            panic!("Wrong config type");
        }
    }
}
