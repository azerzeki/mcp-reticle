//! Event emission utilities for frontend communication
//!
//! This module provides structured event types and emission functions
//! for sending real-time updates to the Tauri frontend via the event system.
//!
//! # Event Types
//! - `log_events`: MCP message interception events
//! - `session_events`: Proxy session lifecycle events
//! - `recording_events`: Session recording status and progress events

pub mod log_events;
pub mod recording_events;
pub mod session_events;

// Re-export commonly used event types and functions
pub use log_events::{emit_log_event, LogEvent};
pub use session_events::{emit_session_start, SessionStartEvent};
