// 存储上下文 - 统一持有仓库实例
use super::sensor_data_repository::SensorDataRepository;
use super::storage_repository::StorageRepository;
use std::sync::Arc;

/// 存储上下文 - 持有所有存储仓库的共享实例
///
/// 由 `StorageFactory` 创建，注入到管道中。
pub struct StorageContext {
    runtime_repo: Arc<dyn StorageRepository>,
    sensor_repo: Arc<dyn SensorDataRepository>,
}

impl StorageContext {
    pub fn new(
        runtime_repo: Arc<dyn StorageRepository>,
        sensor_repo: Arc<dyn SensorDataRepository>,
    ) -> Self {
        Self {
            runtime_repo,
            sensor_repo,
        }
    }

    pub fn runtime_repo(&self) -> Arc<dyn StorageRepository> {
        Arc::clone(&self.runtime_repo)
    }

    pub fn sensor_repo(&self) -> Arc<dyn SensorDataRepository> {
        Arc::clone(&self.sensor_repo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_context_creation() {
        use crate::repositories::MockStorageRepository;

        let runtime_repo: Arc<dyn StorageRepository> = Arc::new(MockStorageRepository::new());

        struct TestSensorRepo;

        #[async_trait::async_trait]
        impl SensorDataRepository for TestSensorRepo {
            async fn save_sensor_data_batch(
                &self,
                _data: &[sensor_core::SensorData],
            ) -> Result<usize, String> {
                Ok(0)
            }
            async fn query_recent_sensor_data(
                &self,
                _limit: usize,
            ) -> Result<Vec<sensor_core::SensorData>, String> {
                Ok(vec![])
            }
            async fn get_latest_sensor_data(
                &self,
            ) -> Result<Option<sensor_core::SensorData>, String> {
                Ok(None)
            }
            async fn get_sensor_data_count(&self) -> Result<i64, String> {
                Ok(0)
            }
            async fn purge_old_sensor_data(&self, _max_records: usize) -> Result<usize, String> {
                Ok(0)
            }
            async fn health_check(&self) -> Result<(), String> {
                Ok(())
            }
        }

        let sensor_repo: Arc<dyn SensorDataRepository> = Arc::new(TestSensorRepo);

        let ctx = StorageContext::new(runtime_repo.clone(), sensor_repo);

        let repo1 = ctx.runtime_repo();
        let repo2 = ctx.runtime_repo();
        assert!(Arc::ptr_eq(&repo1, &repo2));
    }
}
