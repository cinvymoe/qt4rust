// src/pipeline/core/types.rs

use crate::models::ProcessedData;

/// Unified storage events for event-driven architecture
#[derive(Debug, Clone)]
pub enum StorageEvent {
    /// New data available for storage
    NewData(Vec<ProcessedData>),

    /// Alarm triggered
    Alarm(ProcessedData),

    /// Alarm cleared (danger → safe transition)
    AlarmCleared,

    /// Request graceful shutdown
    Shutdown,
}

/// Errors for event-driven storage operations
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Channel send error: {0}")]
    ChannelSend(String),
    #[error("Channel closed")]
    ChannelClosed,
    #[error("Database error: {0}")]
    Database(String),
    #[error("Queue full, would block")]
    QueueFull,
    #[error("Max retries exceeded")]
    MaxRetriesExceeded,
}
