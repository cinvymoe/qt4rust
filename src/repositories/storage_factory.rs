// 存储仓库工厂 - 统一创建存储实例
use super::mock_storage_repository::MockStorageRepository;
use super::sensor_data_repository::SensorDataRepository;
use super::sqlite_storage_repository::SqliteStorageRepository;
use super::storage_context::StorageContext;
use super::storage_repository::StorageRepository;
use std::sync::Arc;

/// 存储仓库工厂
///
/// 统一创建和管理存储仓库实例。通过工厂模式解耦
/// PipelineManager 与具体存储实现（SqliteStorageRepository）的依赖。
pub struct StorageFactory;

impl StorageFactory {
    /// 创建 SQLite 存储上下文（生产环境）
    ///
    /// 使用单一的 SqliteStorageRepository 实例实现了 StorageRepository + SensorDataRepository，
    /// 避免之前两个独立实例导致的数据库连接浪费。
    pub async fn create_sqlite(db_path: &str) -> Result<StorageContext, String> {
        let sqlite_repo = SqliteStorageRepository::new(db_path).await?;
        let sqlite_arc: Arc<SqliteStorageRepository> = Arc::new(sqlite_repo);

        // 同一个 SQLite 实例同时实现两个 trait
        let runtime_repo: Arc<dyn StorageRepository> =
            Arc::clone(&sqlite_arc) as Arc<dyn StorageRepository>;
        let sensor_repo: Arc<dyn SensorDataRepository> =
            Arc::clone(&sqlite_arc) as Arc<dyn SensorDataRepository>;

        tracing::info!("📦 SQLite storage context created: {}", db_path);

        Ok(StorageContext::new(runtime_repo, sensor_repo))
    }

    /// 创建 Mock 存储上下文（测试环境）
    pub fn create_mock() -> StorageContext {
        let mock_repo = Arc::new(MockStorageRepository::new());

        // 同一个 Mock 实例同时实现两个 trait
        let runtime_repo: Arc<dyn StorageRepository> =
            Arc::clone(&mock_repo) as Arc<dyn StorageRepository>;
        let sensor_repo: Arc<dyn SensorDataRepository> =
            Arc::clone(&mock_repo) as Arc<dyn SensorDataRepository>;

        StorageContext::new(runtime_repo, sensor_repo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_mock_context() {
        let ctx = StorageFactory::create_mock();

        let repo1 = ctx.runtime_repo();
        let repo2 = ctx.runtime_repo();
        assert!(Arc::ptr_eq(&repo1, &repo2));
    }
}
