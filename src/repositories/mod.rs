// 数据仓库模块

pub mod crane_data_repository;
pub mod mock_storage_repository;
pub mod sensor_data_repository;
pub mod sqlite_storage_repository;
pub mod storage_repository;

// 重新导出常用类型
pub use crane_data_repository::CraneDataRepository;
pub use mock_storage_repository::MockStorageRepository;
pub use sensor_data_repository::SensorDataRepository;
pub use sqlite_storage_repository::SqliteStorageRepository;
pub use storage_repository::StorageRepository;
