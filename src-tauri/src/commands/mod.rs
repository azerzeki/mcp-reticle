//! Command handlers for Tauri IPC
//!
//! This module contains all Tauri command handlers that can be invoked
//! from the frontend. Commands are grouped by functionality:
//! - `proxy`: Proxy lifecycle management (start, stop, configure)
//! - `demo`: Demo data generation for testing
//! - `recording`: Session recording control and management
//! - `interaction`: Bidirectional MCP communication (send requests)

pub mod demo;
pub mod interaction;
pub mod proxy;
pub mod recording;

// Re-export command functions for use in main.rs
pub use interaction::{can_interact, get_mcp_methods, send_raw_message, send_request};
pub use proxy::{start_proxy, start_proxy_v2, stop_proxy};
pub use recording::{
    delete_recorded_session, export_session, get_recording_status, list_recorded_sessions,
    load_recorded_session, start_recording, stop_recording,
};
