// 存储管道（解耦版本）

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use super::storage_queue::StorageQueue;
use super::shared_buffer::SharedBuffer;
use crate::repositories::storage_repository::StorageRepository;
use crate::models::ProcessedData;

/// 存储管道配置
#[derive(Debug, Clone)]
pub struct StoragePipelineConfig {
    /// 存储间隔（运行数据）
    pub interval: Duration,
    
    /// 批量存储大小
    pub batch_size: usize,
    
    /// 失败重试次数
    pub max_retries: u32,
    
    /// 重试延迟
    pub retry_delay: Duration,
    
    /// 管道队列最大容量
    pub max_queue_size: usize,
}

impl Default for StoragePipelineConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(1),  // 1秒存储一次
            batch_size: 10,
            max_retries: 3,
            retry_delay: Duration::from_millis(100),
            max_queue_size: 1000,  // 最多缓存 1000 条数据
        }
    }
}

/// 存储管道
pub struct StoragePipeline {
    config: StoragePipelineConfig,
    storage_queue: Arc<StorageQueue>,
    repository: Arc<dyn StorageRepository>,  // 依赖抽象接口
    buffer: SharedBuffer,
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
    tokio_runtime: Arc<Runtime>,
}

impl StoragePipeline {
    /// 创建存储管道（依赖注入）
    /// 
    /// # 参数
    /// - `config`: 管道配置
    /// - `repository`: 存储仓库（抽象接口）
    /// - `buffer`: 共享缓冲区
    /// 
    /// # 返回
    /// - `Ok(StoragePipeline)`: 创建成功
    /// - `Err(String)`: 错误信息
    pub fn new(
        config: StoragePipelineConfig,
        repository: Arc<dyn StorageRepository>,
        buffer: SharedBuffer,
    ) -> Result<Self, String> {
        let storage_queue = Arc::new(StorageQueue::new(config.max_queue_size));
        let tokio_runtime = Arc::new(
            Runtime::new().map_err(|e| format!("Failed to create Tokio runtime: {}", e))?
        );
        
        Ok(Self {
            config,
            storage_queue,
            repository,
            buffer,
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
            tokio_runtime,
        })
    }
    
    /// 启动管道
    pub fn start(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            eprintln!("[WARN] Storage pipeline already running");
            return;
        }
        
        self.running.store(true, Ordering::Relaxed);
        
        let config = self.config.clone();
        let storage_queue = Arc::clone(&self.storage_queue);
        let repository = Arc::clone(&self.repository);
        let buffer = Arc::clone(&self.buffer);
        let running = Arc::clone(&self.running);
        let tokio_runtime = Arc::clone(&self.tokio_runtime);
        
        let handle = thread::spawn(move || {
            eprintln!("[INFO] Storage pipeline started");
            
            while running.load(Ordering::Relaxed) {
                let start_time = std::time::Instant::now();
                
                // 1. 从共享缓冲区读取新数据
                if let Ok(buf) = buffer.read() {
                    let last_seq = storage_queue.last_stored_sequence();
                    let new_data = buf.get_history(config.batch_size)
                        .into_iter()
                        .filter(|d| d.sequence_number > last_seq)
                        .collect::<Vec<_>>();
                    
                    for data in new_data {
                        if let Err(e) = storage_queue.push(data) {
                            eprintln!("[ERROR] Failed to push to storage queue: {}", e);
                        }
                    }
                }
                
                // 2. 从队列取出数据批量存储
                let data_to_store = storage_queue.peek_batch(config.batch_size);
                
                if !data_to_store.is_empty() {
                    let max_sequence = data_to_store.iter()
                        .map(|d| d.sequence_number)
                        .max()
                        .unwrap_or(0);
                    
                    // 异步存储到数据库（通过抽象接口）
                    let repository_clone = Arc::clone(&repository);
                    let storage_queue_clone = Arc::clone(&storage_queue);
                    let data_clone = data_to_store.clone();
                    let count = data_to_store.len();
                    
                    tokio_runtime.spawn(async move {
                        // 调用抽象接口方法
                        match repository_clone.save_runtime_data_batch(&data_clone).await {
                            Ok(saved_count) => {
                                eprintln!("[INFO] Saved {} records", saved_count);
                                // 存储成功，从队列删除
                                if let Err(e) = storage_queue_clone.remove_stored(count, max_sequence) {
                                    eprintln!("[ERROR] Failed to remove stored data: {}", e);
                                }
                            }
                            Err(e) => {
                                eprintln!("[ERROR] Failed to save runtime data: {}", e);
                            }
                        }
                    });
                }
                
                // 3. 控制存储频率
                let elapsed = start_time.elapsed();
                if elapsed < config.interval {
                    thread::sleep(config.interval - elapsed);
                }
            }
            
            eprintln!("[INFO] Storage pipeline stopped");
        });
        
        self.handle = Some(handle);
    }
    
    /// 停止管道
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
    
    /// 异步回调：立即存储报警记录（通过抽象接口）
    /// 
    /// # 参数
    /// - `data`: 处理后的数据（包含报警信息）
    pub fn save_alarm_async(&self, data: ProcessedData) {
        let repository = Arc::clone(&self.repository);
        
        self.tokio_runtime.spawn(async move {
            // 调用抽象接口方法
            match repository.save_alarm_record(&data).await {
                Ok(alarm_id) => {
                    eprintln!("[INFO] Alarm saved with id: {}", alarm_id);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to save alarm record: {}", e);
                }
            }
        });
    }
    
    /// 获取存储队列长度
    pub fn queue_len(&self) -> usize {
        self.storage_queue.len()
    }
    
    /// 获取最后存储的序列号
    pub fn last_stored_sequence(&self) -> u64 {
        self.storage_queue.last_stored_sequence()
    }
    
    /// 克隆用于回调（只克隆必要的字段）
    pub fn clone_for_callback(&self) -> Self {
        Self {
            config: self.config.clone(),
            storage_queue: Arc::clone(&self.storage_queue),
            repository: Arc::clone(&self.repository),
            buffer: Arc::clone(&self.buffer),
            running: Arc::clone(&self.running),
            handle: None,  // 不克隆线程句柄
            tokio_runtime: Arc::clone(&self.tokio_runtime),
        }
    }
}

impl Drop for StoragePipeline {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::mock_storage_repository::MockStorageRepository;
    use crate::pipeline::shared_buffer::ProcessedDataBuffer;
    use crate::models::sensor_data::SensorData;
    use std::sync::RwLock;
    
    #[tokio::test]
    async fn test_new() {
        let repo = Arc::new(MockStorageRepository::new());
        let buffer = Arc::new(RwLock::new(ProcessedDataBuffer::new(100)));
        let config = StoragePipelineConfig::default();
        
        let pipeline = StoragePipeline::new(
            config,
            repo as Arc<dyn StorageRepository>,
            buffer,
        );
        
        assert!(pipeline.is_ok());
    }
    
    #[tokio::test]
    async fn test_save_alarm_async() {
        let repo = Arc::new(MockStorageRepository::new());
        let buffer = Arc::new(RwLock::new(ProcessedDataBuffer::new(100)));
        let config = StoragePipelineConfig::default();
        
        let pipeline = StoragePipeline::new(
            config,
            repo.clone() as Arc<dyn StorageRepository>,
            buffer,
        ).unwrap();
        
        // 创建报警数据
        let sensor_data = SensorData::new(23.0, 10.0, 60.0);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);
        
        // 异步保存报警
        pipeline.save_alarm_async(processed);
        
        // 等待异步任务完成
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // 验证报警已保存
        assert_eq!(repo.get_alarm_count(), 1);
    }
    
    #[tokio::test]
    async fn test_queue_operations() {
        let repo = Arc::new(MockStorageRepository::new());
        let buffer = Arc::new(RwLock::new(ProcessedDataBuffer::new(100)));
        let config = StoragePipelineConfig::default();
        
        let pipeline = StoragePipeline::new(
            config,
            repo as Arc<dyn StorageRepository>,
            buffer,
        ).unwrap();
        
        // 初始队列应该为空
        assert_eq!(pipeline.queue_len(), 0);
        assert_eq!(pipeline.last_stored_sequence(), 0);
    }
}
