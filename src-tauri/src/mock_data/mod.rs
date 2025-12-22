//! Mock data generation for testing and demo mode
//!
//! This module provides utilities for generating realistic MCP message
//! sequences for testing the UI and proxy functionality without requiring
//! a real MCP server.

mod generator;

// Re-export mock data type
pub use generator::MockData;
