use serde::{Deserialize, Serialize};
use std::fmt;

/// Transport type for MCP communication
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransportType {
    /// stdio-based transport (stdin/stdout pipes)
    Stdio,
    /// HTTP-based transport with Server-Sent Events
    Http,
}

impl fmt::Display for TransportType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransportType::Stdio => write!(f, "stdio"),
            TransportType::Http => write!(f, "http"),
        }
    }
}

/// Configuration for starting a transport
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum TransportConfig {
    /// stdio transport configuration
    Stdio { command: String, args: Vec<String> },
    /// HTTP/SSE transport configuration
    Http { server_url: String, proxy_port: u16 },
}

impl TransportConfig {
    /// Get the transport type from config
    #[allow(dead_code)]
    pub fn transport_type(&self) -> TransportType {
        match self {
            TransportConfig::Stdio { .. } => TransportType::Stdio,
            TransportConfig::Http { .. } => TransportType::Http,
        }
    }

    /// Check if this is demo mode (empty command or "demo")
    pub fn is_demo(&self) -> bool {
        match self {
            TransportConfig::Stdio { command, .. } => command.is_empty() || command == "demo",
            TransportConfig::Http { .. } => false,
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

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

impl From<TransportError> for String {
    fn from(err: TransportError) -> String {
        err.to_string()
    }
}
