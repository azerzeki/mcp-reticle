//! Application configuration
//!
//! This module defines configuration types and default values:
//! - `app_config`: Configuration structure
//! - `defaults`: Default configuration values

pub mod app_config;
pub mod defaults;

// Re-export configuration type
pub use app_config::AppConfig;
