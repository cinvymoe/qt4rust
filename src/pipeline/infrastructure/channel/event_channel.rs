use crate::models::ProcessedData;
use crate::pipeline::core::StorageError;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, watch};

/// Receiver side for storage events (created by the new StoragePipeline)
#[derive(Clone)]
pub struct StorageEventReceiver {
    pub(crate) data_rx: Arc<Mutex<mpsc::Receiver<Vec<ProcessedData>>>>,
    pub(crate) shutdown_rx: watch::Receiver<bool>,
}

/// Sender side for storage events (used by CollectionPipeline)
#[derive(Clone)]
pub struct StorageEventSender {
    data_tx: mpsc::Sender<Vec<ProcessedData>>,
    shutdown_tx: watch::Sender<bool>,
}

impl StorageEventSender {
    /// Try to send data without waiting (for async contexts)
    pub fn try_send_data(&self, data: Vec<ProcessedData>) -> Result<(), StorageError> {
        self.data_tx.try_send(data).map_err(|e| match e {
            mpsc::error::TrySendError::Full(_) => StorageError::QueueFull,
            mpsc::error::TrySendError::Closed(_) => StorageError::ChannelClosed,
        })
    }

    /// Send shutdown signal
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }
}

impl StorageEventReceiver {
    /// Send shutdown signal
    pub fn shutdown(&self) {
        let _ = self.shutdown_rx.clone();
    }
}

/// Factory for creating connected channel pairs
pub fn create_storage_channels(capacity: usize) -> (StorageEventSender, StorageEventReceiver) {
    let (data_tx, data_rx) = mpsc::channel(capacity);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let sender = StorageEventSender {
        data_tx,
        shutdown_tx,
    };
    let receiver = StorageEventReceiver {
        data_rx: Arc::new(Mutex::new(data_rx)),
        shutdown_rx,
    };

    (sender, receiver)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::ProcessedData;
    use std::time::SystemTime;

    fn create_test_processed_data() -> ProcessedData {
        ProcessedData {
            current_load: 10.0,
            rated_load: 25.0,
            aux_current_load: 0.0,
            aux_moment_percentage: 0.0,
            working_radius: 5.0,
            boom_angle: 45.0,
            boom_length: 10.0,
            moment_percentage: 50.0,
            is_warning: false,
            is_danger: false,
            validation_error: None,
            timestamp: SystemTime::now(),
            sequence_number: 1,
            alarm_sources: Vec::new(),
            alarm_messages: Vec::new(),
        }
    }

    #[test]
    fn test_create_channels() {
        let (sender, _receiver) = create_storage_channels(10);
        assert_eq!(sender.data_tx.capacity(), 10);
    }

    #[tokio::test]
    async fn test_try_send_data_success() {
        let (sender, receiver) = create_storage_channels(10);
        let data = vec![create_test_processed_data()];
        let result = sender.try_send_data(data);
        assert!(result.is_ok());

        let mut rx = receiver.data_rx.lock().await;
        let received = rx.recv().await.unwrap();
        assert_eq!(received.len(), 1);
    }

    #[tokio::test]
    async fn test_try_send_data_queue_full() {
        let (sender, _receiver) = create_storage_channels(1);
        let data = vec![create_test_processed_data()];
        assert!(sender.try_send_data(data).is_ok());

        let result = sender.try_send_data(vec![create_test_processed_data()]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::QueueFull));
    }

    #[tokio::test]
    async fn test_try_send_data_channel_closed() {
        let (sender, receiver) = create_storage_channels(10);
        let receiver_arc = receiver.data_rx.clone();
        drop(receiver);
        drop(receiver_arc);

        let result = sender.try_send_data(vec![create_test_processed_data()]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::ChannelClosed));
    }

    #[tokio::test]
    async fn test_shutdown_signal() {
        let (sender, mut receiver) = create_storage_channels(10);
        sender.shutdown();

        receiver.shutdown_rx.changed().await.unwrap();
        assert!(*receiver.shutdown_rx.borrow());
    }

    #[tokio::test]
    async fn test_sender_clone_and_send() {
        let (sender, receiver) = create_storage_channels(10);
        let sender2 = sender.clone();

        sender
            .try_send_data(vec![create_test_processed_data()])
            .unwrap();
        sender2
            .try_send_data(vec![create_test_processed_data()])
            .unwrap();

        let mut rx = receiver.data_rx.lock().await;
        let count = rx.recv().await.unwrap().len() + rx.recv().await.unwrap().len();
        assert_eq!(count, 2);
    }
}
