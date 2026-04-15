use crate::models::SensorData;
use crate::pipeline::StorageError;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, watch};

/// Receiver side for SensorData storage events
#[derive(Clone)]
pub struct SensorDataEventReceiver {
    pub(super) data_rx: Arc<Mutex<mpsc::Receiver<Vec<SensorData>>>>,
    pub(super) shutdown_rx: watch::Receiver<bool>,
}

/// Sender side for SensorData storage events
#[derive(Clone)]
pub struct SensorDataEventSender {
    data_tx: mpsc::Sender<Vec<SensorData>>,
    shutdown_tx: watch::Sender<bool>,
}

impl SensorDataEventSender {
    /// Try to send data without waiting (for async contexts)
    pub fn try_send_data(&self, data: Vec<SensorData>) -> Result<(), StorageError> {
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

impl SensorDataEventReceiver {
    /// Send shutdown signal
    pub fn shutdown(&self) {
        let _ = self.shutdown_rx.clone();
    }
}

/// Factory for creating connected channel pairs
pub fn create_sensor_data_channels(
    capacity: usize,
) -> (SensorDataEventSender, SensorDataEventReceiver) {
    let (data_tx, data_rx) = mpsc::channel(capacity);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let sender = SensorDataEventSender {
        data_tx,
        shutdown_tx,
    };
    let receiver = SensorDataEventReceiver {
        data_rx: Arc::new(Mutex::new(data_rx)),
        shutdown_rx,
    };

    (sender, receiver)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::SensorData;

    fn create_test_sensor_data() -> SensorData {
        SensorData {
            ad1_load: 10.0,
            ad2_radius: 5.0,
            ad3_angle: 45.0,
        }
    }

    #[test]
    fn test_create_channels() {
        let (sender, _receiver) = create_sensor_data_channels(10);
        assert_eq!(sender.data_tx.capacity(), 10);
    }

    #[tokio::test]
    async fn test_try_send_data_success() {
        let (sender, receiver) = create_sensor_data_channels(10);
        let data = vec![create_test_sensor_data()];
        let result = sender.try_send_data(data);
        assert!(result.is_ok());

        let mut rx = receiver.data_rx.lock().await;
        let received = rx.recv().await.unwrap();
        assert_eq!(received.len(), 1);
    }

    #[tokio::test]
    async fn test_try_send_data_queue_full() {
        let (sender, _receiver) = create_sensor_data_channels(1);
        let data = vec![create_test_sensor_data()];
        assert!(sender.try_send_data(data).is_ok());

        let result = sender.try_send_data(vec![create_test_sensor_data()]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), StorageError::QueueFull));
    }

    #[tokio::test]
    async fn test_shutdown_signal() {
        let (sender, mut receiver) = create_sensor_data_channels(10);
        sender.shutdown();

        receiver.shutdown_rx.changed().await.unwrap();
        assert!(*receiver.shutdown_rx.borrow());
    }
}
