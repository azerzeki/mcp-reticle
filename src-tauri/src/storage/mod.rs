//! Storage layer for session recordings using sled
//!
//! This module provides sled-based persistence for recorded sessions,
//! allowing sessions to be saved, loaded, and queried efficiently.

use crate::core::session_recorder::RecordedSession;
use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use sled::Db;
use std::path::PathBuf;
use std::sync::Arc;

/// Session storage using sled embedded database
pub struct SessionStorage {
    db: Arc<Db>,
}

impl SessionStorage {
    /// Create a new session storage
    pub fn new(db_path: PathBuf) -> Result<Self> {
        let db = sled::open(db_path)
            .map_err(|e| AppError::StorageError(format!("Failed to open sled database: {e}")))?;

        Ok(Self { db: Arc::new(db) })
    }

    /// Save a recorded session
    pub async fn save_session(&self, session: &RecordedSession) -> Result<()> {
        let sessions_tree = self
            .db
            .open_tree("sessions")
            .map_err(|e| AppError::StorageError(format!("Failed to open sessions tree: {e}")))?;

        // Serialize session to bytes
        let session_bytes = bincode::serialize(session).map_err(|e| {
            AppError::SerializationError(format!("Failed to serialize session: {e}"))
        })?;

        // Store with session ID as key
        sessions_tree
            .insert(session.id.as_bytes(), session_bytes)
            .map_err(|e| AppError::StorageError(format!("Failed to insert session: {e}")))?;

        // Also store metadata in index tree for efficient listing
        let index_tree = self
            .db
            .open_tree("session_index")
            .map_err(|e| AppError::StorageError(format!("Failed to open index tree: {e}")))?;

        let info = SessionInfo {
            id: session.id.clone(),
            name: session.name.clone(),
            started_at: session.started_at,
            ended_at: session.ended_at,
            message_count: session.metadata.message_count,
            duration_ms: session.metadata.duration_ms,
            transport: session.metadata.transport.clone(),
        };

        let info_bytes = bincode::serialize(&info)
            .map_err(|e| AppError::SerializationError(format!("Failed to serialize index: {e}")))?;

        // Use timestamp as key for sorted listing
        let key = format!("{:016x}:{}", u64::MAX - session.started_at, session.id);
        index_tree
            .insert(key.as_bytes(), info_bytes)
            .map_err(|e| AppError::StorageError(format!("Failed to insert index: {e}")))?;

        // Flush to disk
        self.db
            .flush_async()
            .await
            .map_err(|e| AppError::StorageError(format!("Failed to flush database: {e}")))?;

        tracing::info!("Saved session {} to sled database", session.id);
        Ok(())
    }

    /// Load a recorded session by ID
    pub async fn load_session(&self, session_id: &str) -> Result<RecordedSession> {
        let sessions_tree = self
            .db
            .open_tree("sessions")
            .map_err(|e| AppError::StorageError(format!("Failed to open sessions tree: {e}")))?;

        let session_bytes = sessions_tree
            .get(session_id.as_bytes())
            .map_err(|e| AppError::StorageError(format!("Failed to get session: {e}")))?
            .ok_or_else(|| AppError::StorageError(format!("Session not found: {session_id}")))?;

        let session: RecordedSession = bincode::deserialize(&session_bytes).map_err(|e| {
            AppError::SerializationError(format!("Failed to deserialize session: {e}"))
        })?;

        Ok(session)
    }

    /// List all recorded sessions (sorted by start time, newest first)
    pub async fn list_sessions(&self) -> Result<Vec<SessionInfo>> {
        let index_tree = self
            .db
            .open_tree("session_index")
            .map_err(|e| AppError::StorageError(format!("Failed to open index tree: {e}")))?;

        let mut sessions = Vec::new();
        let mut seen_ids = std::collections::HashSet::new();

        for item in index_tree.iter() {
            let (_key, value) = item
                .map_err(|e| AppError::StorageError(format!("Failed to iterate sessions: {e}")))?;

            let info: SessionInfo = bincode::deserialize(&value).map_err(|e| {
                AppError::SerializationError(format!("Failed to deserialize index: {e}"))
            })?;

            // Deduplicate by session ID
            if seen_ids.insert(info.id.clone()) {
                sessions.push(info);
            }
        }

        Ok(sessions)
    }

    /// Delete a recorded session
    pub async fn delete_session(&self, session_id: &str) -> Result<()> {
        let sessions_tree = self
            .db
            .open_tree("sessions")
            .map_err(|e| AppError::StorageError(format!("Failed to open sessions tree: {e}")))?;

        // Remove from sessions tree
        sessions_tree
            .remove(session_id.as_bytes())
            .map_err(|e| AppError::StorageError(format!("Failed to remove session: {e}")))?;

        // Remove from index tree
        let index_tree = self
            .db
            .open_tree("session_index")
            .map_err(|e| AppError::StorageError(format!("Failed to open index tree: {e}")))?;

        // Find and remove index entry
        let mut key_to_remove = None;
        for item in index_tree.iter() {
            let (key, value) =
                item.map_err(|e| AppError::StorageError(format!("Failed to iterate index: {e}")))?;

            let info: SessionInfo = bincode::deserialize(&value).map_err(|e| {
                AppError::SerializationError(format!("Failed to deserialize index: {e}"))
            })?;

            if info.id == session_id {
                key_to_remove = Some(key.to_vec());
                break;
            }
        }

        if let Some(key) = key_to_remove {
            index_tree
                .remove(key)
                .map_err(|e| AppError::StorageError(format!("Failed to remove index: {e}")))?;
        }

        // Flush to disk
        self.db
            .flush_async()
            .await
            .map_err(|e| AppError::StorageError(format!("Failed to flush database: {e}")))?;

        tracing::info!("Deleted session {}", session_id);
        Ok(())
    }

    /// Get storage statistics
    #[allow(dead_code)]
    pub fn get_stats(&self) -> Result<StorageStats> {
        let sessions_tree = self
            .db
            .open_tree("sessions")
            .map_err(|e| AppError::StorageError(format!("Failed to open sessions tree: {e}")))?;

        let session_count = sessions_tree.len();
        let db_size = self
            .db
            .size_on_disk()
            .map_err(|e| AppError::StorageError(format!("Failed to get database size: {e}")))?;

        Ok(StorageStats {
            session_count,
            size_bytes: db_size,
        })
    }
}

/// Session information for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub name: String,
    pub started_at: u64,
    pub ended_at: Option<u64>,
    pub message_count: usize,
    pub duration_ms: Option<u64>,
    pub transport: String,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub session_count: usize,
    pub size_bytes: u64,
}

// bincode support - add to dependencies
mod bincode {
    use serde::{Deserialize, Serialize};

    pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
        serde_json::to_vec(value).map_err(|e| e.to_string())
    }

    pub fn deserialize<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, String> {
        serde_json::from_slice(bytes).map_err(|e| e.to_string())
    }
}
