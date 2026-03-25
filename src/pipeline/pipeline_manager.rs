// 管道管理器 - 统一管理三个管道

use std::sync::Arc;
use crate::repositories::CraneDataRepository;
use crate::repositories::storage_repository::StorageRepository;
use crate::repositories::sqlite_storage_repository::SqliteStorageRepository;
use super::shared_buffer::{ProcessedDataBuffer, SharedBuffer};
use super::collection_pipeline::{CollectionPipeline, CollectionPipelineConfig};
use super::storage_pipeline::{StoragePipeline, StoragePipelineConfig};

/// 管道管理器
pub struct PipelineManager {
    /// 采集管道
    collection_pipeline: Option<CollectionPipeline>,
    
    /// 存储管道
    storage_pipeline: Option<StoragePipeline>,
    
    /// 共享缓冲区
    shared_buffer: SharedBuffer,
    
    /// 数据仓库
    repository: Arc<CraneDataRepository>,
}

impl PipelineManager {
    /// 创建管道管理器
    pub fn new(repository: Arc<CraneDataRepository>) -> Self {
        // 创建共享缓冲区（保留最近 1000 条数据）
        let shared_buffer = Arc::new(std::sync::RwLock::new(
            ProcessedDataBuffer::new(1000)
        ));
        
        Self {
            collection_pipeline: None,
            storage_pipeline: None,
            shared_buffer,
            repository,
        }
    }
    
    /// 创建管道管理器（带数据库路径）
    /// 
    /// # 参数
    /// - `repository`: 数据仓库
    /// - `db_path`: SQLite 数据库路径
    /// 
    /// # 返回
    /// - `Ok(PipelineManager)`: 创建成功
    /// - `Err(String)`: 错误信息
    pub async fn new_with_storage(
        repository: Arc<CraneDataRepository>,
        db_path: &str,
    ) -> Result<Self, String> {
        // 创建共享缓冲区（保留最近 1000 条数据）
        let shared_buffer = Arc::new(std::sync::RwLock::new(
            ProcessedDataBuffer::new(1000)
        ));
        
        // 创建存储仓库
        let storage_repo = SqliteStorageRepository::new(db_path).await?;
        
        // 从配置文件加载存储配置
        let pipeline_config = crate::config::pipeline_config::PipelineConfig::load();
        let config = StoragePipelineConfig::from_pipeline_config(&pipeline_config.storage);
        
        eprintln!("[INFO] Storage pipeline config loaded:");
        eprintln!("  - Interval: {}ms", config.interval.as_millis());
        eprintln!("  - Batch size: {}", config.batch_size);
        eprintln!("  - Max retries: {}", config.max_retries);
        eprintln!("  - Retry delay: {}ms", config.retry_delay.as_millis());
        eprintln!("  - Max queue size: {}", config.max_queue_size);
        
        // 创建存储管道
        let storage_pipeline = StoragePipeline::new(
            config,
            Arc::new(storage_repo) as Arc<dyn StorageRepository>,
            Arc::clone(&shared_buffer),
        )?;
        
        Ok(Self {
            collection_pipeline: None,
            storage_pipeline: Some(storage_pipeline),
            shared_buffer,
            repository,
        })
    }
    
    /// 启动采集管道（后台线程 1）
    pub fn start_collection_pipeline(&mut self) {
        if self.collection_pipeline.is_some() {
            eprintln!("[WARN] Collection pipeline already started");
            return;
        }
        
        eprintln!("[INFO] Starting collection pipeline (Backend Thread 1)...");
        
        // 创建配置
        let config = CollectionPipelineConfig::default();
        
        // 创建采集管道
        let mut pipeline = CollectionPipeline::new(
            config,
            Arc::clone(&self.repository),
            Arc::clone(&self.shared_buffer),
        );
        
        // 如果存储管道已启动，设置报警回调
        if let Some(storage_pipeline) = &self.storage_pipeline {
            let storage_clone = storage_pipeline.clone_for_callback();
            pipeline.set_alarm_callback(Box::new(move |data| {
                storage_clone.save_alarm_async(data);
            }));
            eprintln!("[INFO] Alarm callback connected to storage pipeline");
        }
        
        // 启动管道
        pipeline.start();
        
        self.collection_pipeline = Some(pipeline);
        
        eprintln!("[INFO] Collection pipeline started successfully");
        eprintln!("[INFO] - Interval: 100ms (10Hz)");
        eprintln!("[INFO] - Max retries: 3");
        eprintln!("[INFO] - Disconnect threshold: 10");
    }
    
    /// 停止采集管道
    pub fn stop_collection_pipeline(&mut self) {
        if let Some(mut pipeline) = self.collection_pipeline.take() {
            eprintln!("[INFO] Stopping collection pipeline...");
            pipeline.stop();
            eprintln!("[INFO] Collection pipeline stopped");
        }
    }
    
    /// 启动存储管道（后台线程 2）
    pub fn start_storage_pipeline(&mut self) {
        if let Some(pipeline) = &mut self.storage_pipeline {
            eprintln!("[INFO] Starting storage pipeline (Backend Thread 2)...");
            pipeline.start();
            eprintln!("[INFO] Storage pipeline started successfully");
        } else {
            eprintln!("[WARN] Storage pipeline not initialized. Use new_with_storage() to create manager with storage support.");
        }
    }
    
    /// 停止存储管道
    pub fn stop_storage_pipeline(&mut self) {
        if let Some(pipeline) = &mut self.storage_pipeline {
            eprintln!("[INFO] Stopping storage pipeline...");
            pipeline.stop();
            eprintln!("[INFO] Storage pipeline stopped");
        }
    }
    
    /// 启动所有管道
    pub fn start_all(&mut self) {
        eprintln!("[INFO] Starting all pipelines...");
        
        // 先启动存储管道
        self.start_storage_pipeline();
        
        // 再启动采集管道（会自动连接报警回调）
        self.start_collection_pipeline();
        
        // TODO: 启动显示管道（主线程）
        eprintln!("[INFO] All pipelines started");
    }
    
    /// 停止所有管道
    pub fn stop_all(&mut self) {
        eprintln!("[INFO] Stopping all pipelines...");
        
        // 先停止采集管道
        self.stop_collection_pipeline();
        
        // 再停止存储管道
        self.stop_storage_pipeline();
        
        // TODO: 停止显示管道
        eprintln!("[INFO] All pipelines stopped");
    }
    
    /// 获取共享缓冲区（用于调试和显示管道）
    pub fn get_shared_buffer(&self) -> SharedBuffer {
        Arc::clone(&self.shared_buffer)
    }
    
    /// 检查采集管道是否运行中
    pub fn is_collection_running(&self) -> bool {
        self.collection_pipeline.is_some()
    }
    
    /// 检查存储管道是否运行中
    pub fn is_storage_running(&self) -> bool {
        self.storage_pipeline.is_some()
    }
    
    /// 获取存储队列长度
    pub fn get_storage_queue_len(&self) -> Option<usize> {
        self.storage_pipeline.as_ref().map(|p| p.queue_len())
    }
    
    /// 获取最后存储的序列号
    pub fn get_last_stored_sequence(&self) -> Option<u64> {
        self.storage_pipeline.as_ref().map(|p| p.last_stored_sequence())
    }
}

impl Drop for PipelineManager {
    fn drop(&mut self) {
        self.stop_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_pipeline_manager_creation() {
        let repository = Arc::new(CraneDataRepository::default());
        let manager = PipelineManager::new(repository);
        
        assert!(!manager.is_collection_running());
    }
    
    #[test]
    fn test_start_stop_collection() {
        let repository = Arc::new(CraneDataRepository::default());
        let mut manager = PipelineManager::new(repository);
        
        // 启动
        manager.start_collection_pipeline();
        assert!(manager.is_collection_running());
        
        // 等待一小段时间让管道运行
        std::thread::sleep(std::time::Duration::from_millis(300));
        
        // 检查缓冲区是否有数据
        let buffer = manager.get_shared_buffer();
        let has_data = buffer.read().unwrap().get_latest().is_some();
        assert!(has_data, "Buffer should have data after collection");
        
        // 停止
        manager.stop_collection_pipeline();
        assert!(!manager.is_collection_running());
    }
    
    #[test]
    fn test_data_collection_frequency() {
        let repository = Arc::new(CraneDataRepository::default());
        let mut manager = PipelineManager::new(repository);
        
        // 启动采集
        manager.start_collection_pipeline();
        
        // 等待 1 秒（应该采集约 10 次数据，因为频率是 10Hz）
        std::thread::sleep(std::time::Duration::from_secs(1));
        
        // 检查统计信息
        let buffer = manager.get_shared_buffer();
        let stats = buffer.read().unwrap().get_stats().clone();
        
        eprintln!("[TEST] Collection stats after 1 second:");
        eprintln!("  - Total collections: {}", stats.total_collections);
        eprintln!("  - Success count: {}", stats.success_count);
        eprintln!("  - Error count: {}", stats.error_count);
        
        // 应该至少采集了 8 次（考虑启动延迟）
        assert!(stats.total_collections >= 8, 
                "Should collect at least 8 times in 1 second at 10Hz, got {}", 
                stats.total_collections);
        
        // 成功率应该很高
        assert!(stats.success_count >= 8, 
                "Should have at least 8 successful collections");
        
        manager.stop_collection_pipeline();
    }
    
    #[test]
    fn test_processed_data_content() {
        let repository = Arc::new(CraneDataRepository::default());
        let mut manager = PipelineManager::new(repository);
        
        // 启动采集
        manager.start_collection_pipeline();
        
        // 等待数据采集
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        // 获取最新数据
        let buffer = manager.get_shared_buffer();
        let latest = buffer.read().unwrap().get_latest();
        
        assert!(latest.is_some(), "Should have collected data");
        
        let data = latest.unwrap();
        eprintln!("[TEST] Processed data:");
        eprintln!("  - Sequence: {}", data.sequence_number);
        eprintln!("  - Load: {:.2} tons", data.current_load);
        eprintln!("  - Radius: {:.2} m", data.working_radius);
        eprintln!("  - Angle: {:.2}°", data.boom_angle);
        eprintln!("  - Moment %: {:.2}%", data.moment_percentage);
        eprintln!("  - Is danger: {}", data.is_danger);
        
        // 验证数据合理性
        assert!(data.current_load >= 0.0, "Load should be non-negative");
        assert!(data.working_radius >= 0.0, "Radius should be non-negative");
        assert!(data.boom_angle >= 0.0 && data.boom_angle <= 90.0, 
                "Angle should be between 0 and 90 degrees");
        assert!(data.moment_percentage >= 0.0, "Moment percentage should be non-negative");
        
        manager.stop_collection_pipeline();
    }
    
    #[test]
    fn test_history_buffer() {
        let repository = Arc::new(CraneDataRepository::default());
        let mut manager = PipelineManager::new(repository);
        
        // 启动采集
        manager.start_collection_pipeline();
        
        // 等待采集多条数据
        std::thread::sleep(std::time::Duration::from_millis(500));
        
        // 获取历史数据
        let buffer = manager.get_shared_buffer();
        let history = buffer.read().unwrap().get_history(10);
        
        eprintln!("[TEST] History buffer contains {} records", history.len());
        
        // 应该有多条历史记录
        assert!(history.len() >= 3, "Should have at least 3 historical records");
        
        // 验证序列号递增
        for i in 0..history.len().saturating_sub(1) {
            assert!(history[i].sequence_number > history[i + 1].sequence_number,
                    "History should be in reverse chronological order");
        }
        
        manager.stop_collection_pipeline();
    }
}
