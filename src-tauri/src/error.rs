//! Application-wide error types
//!
//! This module defines a centralized error type using `thiserror` for
//! clean error handling across the application.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Application-wide error type
///
/// This provides a centralized error handling strategy with:
/// - Structured error variants for different failure modes
/// - Serde support for sending errors to frontend
/// - Automatic Display implementation via thiserror
/// - Automatic conversion from common error types
#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[serde(tag = "type", content = "message")]
pub enum AppError {
    /// Proxy is already running
    #[error("Proxy is already running")]
    ProxyAlreadyRunning,

    /// Proxy is not running
    #[error("Proxy is not running")]
    ProxyNotRunning,

    /// Failed to start proxy process
    #[error("Failed to start proxy: {0}")]
    ProxyStartFailed(String),

    /// Failed to emit event to frontend
    #[error("Failed to emit event: {0}")]
    EventEmissionFailed(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Storage/database error
    #[error("Storage error: {0}")]
    StorageError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(String),

    /// JSON serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Generic error with custom message
    #[error("{0}")]
    Other(String),
}

/// Convert AppError to String for Tauri commands
/// Tauri commands require Result<T, String> or custom serializable errors
impl From<AppError> for String {
    fn from(error: AppError) -> String {
        error.to_string()
    }
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, AppError>;

// Automatic conversions from common error types
impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        Self::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

impl From<tauri::Error> for AppError {
    fn from(err: tauri::Error) -> Self {
        Self::EventEmissionFailed(err.to_string())
    }
}
