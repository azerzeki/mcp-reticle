//! Core Reticle functionality
//!
//! This module contains the core proxy implementation and transport abstractions:
//! - `protocol`: MCP protocol message parsing and validation
//! - `proxy`: stdio-based proxy implementation
//! - `sse_proxy`: HTTP/SSE-based proxy implementation
//! - `transport`: Transport configuration and error types
//! - `session_recorder`: Session recording and replay

pub mod protocol;
pub mod proxy;
pub mod session_recorder;
pub mod sse_proxy;
pub mod transport;

// Re-export core types and functions
pub use proxy::run_proxy;
pub use session_recorder::SessionRecorder;
pub use sse_proxy::start_sse_proxy;
pub use transport::TransportConfig;
