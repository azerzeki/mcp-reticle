//! Application state management
//!
//! This module contains the application-wide state types that are managed
//! by Tauri and shared across commands:
//! - `app_state`: Main application state including config and proxy state
//! - `proxy_state`: Proxy lifecycle state (running, stopped, session tracking)

pub mod app_state;
pub mod proxy_state;

// Re-export state types
pub use app_state::AppState;
pub use proxy_state::ProxyState;
