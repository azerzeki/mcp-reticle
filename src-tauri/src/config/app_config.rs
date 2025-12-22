use serde::{Deserialize, Serialize};

/// Application configuration
///
/// Provides centralized configuration management with:
/// - Serde support for loading from files/env
/// - Builder pattern for customization
/// - Sensible defaults
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Demo mode settings
    pub demo: DemoConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemoConfig {
    /// Session ID to use for demo mode
    pub session_id: String,

    /// Delay between emitting messages (milliseconds)
    pub message_delay_ms: u64,

    /// How often to log progress (every N messages)
    pub progress_batch_size: usize,

    /// Delay before starting to emit messages (milliseconds)
    pub startup_delay_ms: u64,
}

impl Default for DemoConfig {
    fn default() -> Self {
        Self {
            session_id: super::defaults::DEFAULT_DEMO_SESSION_ID.to_string(),
            message_delay_ms: super::defaults::DEFAULT_DEMO_MESSAGE_DELAY_MS,
            progress_batch_size: super::defaults::DEFAULT_DEMO_PROGRESS_BATCH,
            startup_delay_ms: super::defaults::DEFAULT_SESSION_STARTUP_DELAY_MS,
        }
    }
}

impl AppConfig {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder method for demo configuration
    #[allow(dead_code)]
    pub fn with_demo_config(mut self, config: DemoConfig) -> Self {
        self.demo = config;
        self
    }
}
