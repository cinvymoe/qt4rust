use crate::error::StorageError;
use crate::pipeline::aggregator::AggregatedSensorData;
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Repository trait for storing and retrieving aggregated sensor data.
#[async_trait]
pub trait StorageRepository: Send + Sync {
    /// Save a batch of aggregated sensor data.
    async fn save_aggregated_data_batch(
        &self,
        data: Vec<AggregatedSensorData>,
    ) -> Result<(), StorageError>;

    /// Query recent aggregated sensor data.
    async fn query_recent_aggregated_data(
        &self,
        limit: usize,
    ) -> Result<Vec<AggregatedSensorData>, StorageError>;

    /// Check storage health.
    async fn health_check(&self) -> Result<(), StorageError>;

    /// Get the last sequence number from storage.
    async fn get_last_sequence(&self) -> Result<u64, StorageError>;
}

/// Mock implementation of StorageRepository for testing.
pub struct MockStorageRepository {
    data: Arc<Mutex<Vec<AggregatedSensorData>>>,
}

impl MockStorageRepository {
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get a copy of all stored data for verification in tests.
    pub async fn get_stored_data(&self) -> Vec<AggregatedSensorData> {
        self.data.lock().await.clone()
    }
}

impl Default for MockStorageRepository {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageRepository for MockStorageRepository {
    async fn save_aggregated_data_batch(
        &self,
        data: Vec<AggregatedSensorData>,
    ) -> Result<(), StorageError> {
        let mut stored = self.data.lock().await;
        stored.extend(data);
        Ok(())
    }

    async fn query_recent_aggregated_data(
        &self,
        limit: usize,
    ) -> Result<Vec<AggregatedSensorData>, StorageError> {
        let stored = self.data.lock().await;
        let count = stored.len().min(limit);
        Ok(stored[stored.len().saturating_sub(count)..].to_vec())
    }

    async fn health_check(&self) -> Result<(), StorageError> {
        Ok(())
    }

    async fn get_last_sequence(&self) -> Result<u64, StorageError> {
        let stored = self.data.lock().await;
        Ok(stored.len() as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::data_source::DataSourceId;
    use std::collections::HashMap;

    fn create_test_data(weight: f64) -> AggregatedSensorData {
        let mut sources = HashMap::new();
        sources.insert(
            DataSourceId::Simulator,
            crate::data::sensor_data::SensorData::new(weight, 50.0, 45.0),
        );
        AggregatedSensorData::new(sources)
    }

    #[tokio::test]
    async fn test_save_and_query() {
        let repo = MockStorageRepository::new();
        let data = vec![create_test_data(100.0), create_test_data(200.0)];

        repo.save_aggregated_data_batch(data.clone())
            .await
            .unwrap();

        let result = repo.query_recent_aggregated_data(10).await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_query_with_limit() {
        let repo = MockStorageRepository::new();
        let data = vec![
            create_test_data(100.0),
            create_test_data(200.0),
            create_test_data(300.0),
        ];

        repo.save_aggregated_data_batch(data).await.unwrap();

        let result = repo.query_recent_aggregated_data(2).await.unwrap();
        assert_eq!(result.len(), 2);
    }

    #[tokio::test]
    async fn test_health_check() {
        let repo = MockStorageRepository::new();
        assert!(repo.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_get_last_sequence() {
        let repo = MockStorageRepository::new();

        assert_eq!(repo.get_last_sequence().await.unwrap(), 0);

        repo.save_aggregated_data_batch(vec![create_test_data(100.0)])
            .await
            .unwrap();
        assert_eq!(repo.get_last_sequence().await.unwrap(), 1);

        repo.save_aggregated_data_batch(vec![
            create_test_data(200.0),
            create_test_data(300.0),
        ])
        .await
        .unwrap();
        assert_eq!(repo.get_last_sequence().await.unwrap(), 3);
    }

    #[tokio::test]
    async fn test_get_stored_data() {
        let repo = MockStorageRepository::new();
        let data = vec![create_test_data(100.0), create_test_data(200.0)];

        repo.save_aggregated_data_batch(data.clone())
            .await
            .unwrap();

        let stored = repo.get_stored_data().await;
        assert_eq!(stored.len(), 2);
    }
}
