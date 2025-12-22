//! Recording events for UI updates
//!
//! This module provides events for notifying the frontend about
//! recording state changes and progress.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

use crate::error::{AppError, Result};

/// Recording started event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStartedEvent {
    pub session_id: String,
    pub session_name: String,
    pub started_at: u64,
}

/// Recording stopped event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingStoppedEvent {
    pub session_id: String,
    pub message_count: usize,
    pub duration_ms: u64,
}

/// Recording message captured event (for live updates)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordingMessageEvent {
    pub session_id: String,
    pub message_count: usize,
    pub direction: String, // "to_server" or "to_client"
}

/// Emit a recording-started event to the frontend
pub fn emit_recording_started(app_handle: &AppHandle, event: RecordingStartedEvent) -> Result<()> {
    app_handle
        .emit("recording-started", event)
        .map_err(|e| AppError::EventEmissionFailed(e.to_string()))
}

/// Emit a recording-stopped event to the frontend
pub fn emit_recording_stopped(app_handle: &AppHandle, event: RecordingStoppedEvent) -> Result<()> {
    app_handle
        .emit("recording-stopped", event)
        .map_err(|e| AppError::EventEmissionFailed(e.to_string()))
}

/// Emit a recording-message event to the frontend
pub fn emit_recording_message(app_handle: &AppHandle, event: RecordingMessageEvent) -> Result<()> {
    app_handle
        .emit("recording-message", event)
        .map_err(|e| AppError::EventEmissionFailed(e.to_string()))
}
