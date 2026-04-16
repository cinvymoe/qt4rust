// 管道管理器 - 统一管理多个管道

use crate::pipeline::infrastructure::{
    create_sensor_data_channels, create_storage_channels,
    EventBus, ProcessedDataBuffer, SharedBuffer, SharedSensorBuffer, StorageEventSender,
};
use crate::pipeline::pipelines::{
    CollectionPipeline, CollectionPipelineConfig, DisplayPipeline, DisplayPipelineConfig,
    ProcessPipeline, ProcessPipelineConfig, SensorStoragePipeline, StoragePipeline,
    StoragePipelineConfig,
};
use crate::pipeline::services::{
    CalibrationService, ConfigProvider, FilterBuffer, FilterBufferConfig, StorageService,
};
use crate::models::crane_config::CraneConfig;
use crate::models::rated_load_table::RatedLoadTable;
use crate::repositories::storage_factory::StorageFactory;
use crate::repositories::CraneDataRepository;
use sensor_core::{AlarmThresholds, SensorCalibration};
use std::sync::Arc;
use std::sync::{Mutex, RwLock};
use tracing::{debug, error, info, trace, warn};

pub struct PipelineManager {
    collection_pipeline: Option<CollectionPipeline>,
    process_pipeline: Option<ProcessPipeline>,
    storage_pipeline: Option<StoragePipeline>,
    display_pipeline: Option<DisplayPipeline>,
    shared_buffer: SharedBuffer,
    #[allow(dead_code)]
    filter_buffer: Option<Arc<Mutex<FilterBuffer>>>,
    repository: Arc<CraneDataRepository>,
    #[allow(dead_code)]
    crane_config: Arc<CraneConfig>,
    #[allow(dead_code)]
    storage_event_sender: Option<StorageEventSender>,
    sensor_storage_pipeline: Option<SensorStoragePipeline>,
    shared_sensor_buffer: Option<SharedSensorBuffer>,
    // 热重载配置引用
    sensor_calibration: Option<Arc<RwLock<SensorCalibration>>>,
    rated_load_table: Option<Arc<RwLock<RatedLoadTable>>>,
    // NEW: Unified configuration provider
    config_provider: Option<Arc<ConfigProvider>>,
    // NEW: Calibration service for AD conversion
    calibration_service: Option<CalibrationService>,
    // NEW: Event bus for unified event dispatch
    event_bus: Option<EventBus>,
}

impl PipelineManager {
    pub fn new(repository: Arc<CraneDataRepository>) -> Self {
        info!("创建管道管理器");
        let shared_buffer = Arc::new(std::sync::RwLock::new(ProcessedDataBuffer::new(1000)));
        debug!("共享缓冲区已创建，容量: 1000");

        // Create ConfigProvider
        let config_provider = Some(Arc::new(ConfigProvider::new()));

        // Create CalibrationService from ConfigProvider
        let calibration_service = config_provider
            .as_ref()
            .map(|cp| CalibrationService::from_provider(cp.as_ref()));

        Self {
            collection_pipeline: None,
            process_pipeline: None,
            storage_pipeline: None,
            display_pipeline: None,
            shared_buffer,
            filter_buffer: None,
            repository,
            crane_config: Arc::new(CraneConfig::default()),
            storage_event_sender: None,
            sensor_storage_pipeline: None,
            shared_sensor_buffer: None,
            sensor_calibration: None,
            rated_load_table: None,
            // New fields
            config_provider,
            calibration_service,
            event_bus: None,
        }
    }

    /// 创建管道管理器（带数据库路径）
    ///
    /// # 参数
    /// - `repository`: 数据仓库
    /// - `db_path`: SQLite 数据库路径
    pub async fn new_with_storage(
        repository: Arc<CraneDataRepository>,
        db_path: &str,
    ) -> Result<Self, String> {
        let shared_buffer = Arc::new(std::sync::RwLock::new(ProcessedDataBuffer::new(1000)));

        // Create ConfigProvider and CalibrationService
        let config_provider = Arc::new(ConfigProvider::new());
        let calibration_service = CalibrationService::from_provider(&config_provider);

        let storage_context = StorageFactory::create_sqlite(db_path).await?;

        let pipeline_config = crate::config::pipeline_config::PipelineConfig::load();
        let (storage_pipeline_config, service_config) =
            StoragePipelineConfig::from_pipeline_config(&pipeline_config.storage);

        info!(
            "Storage pipeline: interval={}ms, batch_size={}",
            storage_pipeline_config.interval.as_millis(),
            storage_pipeline_config.batch_size
        );

        let (storage_sender, storage_receiver) =
            create_storage_channels(storage_pipeline_config.max_queue_size);

        // Create EventBus from existing channels
        let event_bus = Some(EventBus::storage_only(storage_sender.clone()));

        let service = Arc::new(StorageService::new(
            storage_context.runtime_repo(),
            service_config,
        ));

        let mut storage_pipeline = StoragePipeline::with_event_channel(
            storage_pipeline_config,
            Arc::clone(&service),
            storage_receiver,
        );

        storage_pipeline.initialize_sequence().await?;

        let crane_config = Arc::new(CraneConfig::default());

        let filter_config = FilterBufferConfig::from_str(
            &pipeline_config.filter.filter_type,
            pipeline_config.filter.window_size,
        )
        .map_err(|e| format!("Invalid filter config: {}", e))?;

        let fb = Arc::new(Mutex::new(FilterBuffer::new(filter_config)));

        let process_config = ProcessPipelineConfig {
            interval: pipeline_config.process_interval(),
        };
        let process_pipeline = ProcessPipeline::with_event_sender(
            process_config,
            Arc::clone(&fb),
            Arc::clone(&shared_buffer),
            Arc::clone(&crane_config),
            storage_sender.clone(),
        );

        let collection_interval = pipeline_config.collection_interval();
        debug!(
            "Collection pipeline config: interval={}ms from TOML (interval_ms={})",
            collection_interval.as_millis(),
            pipeline_config.collection.interval_ms
        );

        let collection_config = CollectionPipelineConfig {
            interval: collection_interval,
            max_retries: 3,
            retry_delay: std::time::Duration::from_millis(10),
            disconnect_threshold: 10,
            enable_panic_recovery: true,
            max_restarts: 5,
        };

        // Check if sensor storage is enabled
        let (collection_pipeline, sensor_storage_pipeline, shared_sensor_buffer) =
            if pipeline_config.sensor_storage.enabled {
                let sensor_repo = storage_context.sensor_repo();
                let (sensor_tx, sensor_rx) = create_sensor_data_channels(1000);

                let sensor_pipeline = SensorStoragePipeline::with_event_channel(
                    pipeline_config.sensor_storage.clone(),
                    sensor_repo,
                    sensor_rx,
                );
                // 不在这里启动，让 start_all() 统一启动所有管道

                let sensor_buffer = Arc::new(std::sync::RwLock::new(
                    crate::pipeline::infrastructure::SensorDataBuffer::new(100),
                ));

                // Use with_storage_sender when sensor storage is enabled
                let collection = CollectionPipeline::with_storage_sender(
                    collection_config,
                    Arc::clone(&repository),
                    Arc::clone(&shared_buffer),
                    storage_sender.clone(),
                    sensor_tx,
                    sensor_buffer.clone(),
                );

                (Some(collection), Some(sensor_pipeline), Some(sensor_buffer))
            } else {
                let collection = CollectionPipeline::with_filter_buffer(
                    collection_config,
                    Arc::clone(&repository),
                    Arc::clone(&fb),
                );
                (Some(collection), None, None)
            };

        info!("Pipeline: collection={}ms, filter={}/{}, process={}ms, storage={}ms, sensor_storage={}",
            pipeline_config.collection.interval_ms,
            pipeline_config.filter.filter_type,
            pipeline_config.filter.window_size,
            pipeline_config.process.interval_ms,
            pipeline_config.storage.interval_ms,
            pipeline_config.sensor_storage.enabled);

        Ok(Self {
            collection_pipeline,
            process_pipeline: Some(process_pipeline),
            storage_pipeline: Some(storage_pipeline),
            display_pipeline: None,
            shared_buffer,
            filter_buffer: if sensor_storage_pipeline.is_some() {
                None
            } else {
                Some(fb)
            },
            repository,
            crane_config,
            storage_event_sender: Some(storage_sender),
            sensor_storage_pipeline,
            shared_sensor_buffer,
            sensor_calibration: None,
            rated_load_table: None,
            config_provider: Some(config_provider),
            calibration_service: Some(calibration_service),
            event_bus,
        })
    }

    /// 设置热重载配置引用
    pub fn set_hot_reload_config(
        &mut self,
        sensor_calibration: Arc<RwLock<SensorCalibration>>,
        rated_load_table: Arc<RwLock<RatedLoadTable>>,
        alarm_thresholds: Arc<RwLock<AlarmThresholds>>,
    ) {
        if let Some(ref provider) = self.config_provider {
            provider.update_all(
                sensor_calibration.read().unwrap().clone(),
                rated_load_table.read().unwrap().clone(),
                alarm_thresholds.read().unwrap().clone(),
            );
            info!("Hot reload config updated via ConfigProvider");
        }

        self.sensor_calibration = Some(sensor_calibration.clone());
        self.rated_load_table = Some(rated_load_table.clone());

        // 将配置传递给ProcessPipeline
        if let Some(ref mut pp) = self.process_pipeline {
            pp.set_hot_reload_config(
                sensor_calibration.clone(),
                rated_load_table.clone(),
                alarm_thresholds.clone(),
            );
            info!("🔗 [PipelineManager] 热重载配置已设置到ProcessPipeline");
        } else {
            warn!("⚠️  [PipelineManager] ProcessPipeline不存在，无法设置热重载配置");
        }

        // 将配置传递给CollectionPipeline
        if let Some(ref mut cp) = self.collection_pipeline {
            cp.set_hot_reload_config(sensor_calibration, rated_load_table, alarm_thresholds);
            info!("🔗 [PipelineManager] 热重载配置已设置到CollectionPipeline");
        } else {
            warn!("⚠️  [PipelineManager] CollectionPipeline不存在，无法设置热重载配置");
        }
    }

    /// 启动采集管道（后台线程 1）
    pub fn start_collection_pipeline(&mut self) {
        if self.collection_pipeline.is_some() {
            // Pipeline already exists (from new_with_storage), just start it
            info!("启动采集管道（后台线程 1）...");

            if let Some(ref mut pipeline) = self.collection_pipeline {
                // 从存储管道获取最后存储的序列号
                if let Some(storage_pipeline) = &self.storage_pipeline {
                    let last_seq = storage_pipeline.last_stored_sequence();
                    if last_seq > 0 {
                        pipeline.set_initial_sequence(last_seq);
                        info!("采集管道序列号初始化为: {}", last_seq);
                    }
                }

                pipeline.start();
                info!("采集管道启动成功 - 频率: 10Hz (100ms), 最大重试: 3, 断连阈值: 10");
            }
            return;
        }

        info!("启动采集管道（后台线程 1）...");

        // 创建配置（legacy mode - no storage connection）
        let config = CollectionPipelineConfig::default();
        debug!(
            "采集管道配置: interval={}ms, max_retries={}",
            config.interval.as_millis(),
            config.max_retries
        );

        // 创建采集管道（无存储连接，legacy模式）
        let mut pipeline = CollectionPipeline::new(
            config,
            Arc::clone(&self.repository),
            Arc::clone(&self.shared_buffer),
        );

        // 启动管道
        pipeline.start();
        self.collection_pipeline = Some(pipeline);

        info!("采集管道启动成功（无存储连接）- 频率: 10Hz (100ms), 最大重试: 3, 断连阈值: 10");
    }

    /// 停止采集管道
    pub fn stop_collection_pipeline(&mut self) {
        if let Some(mut pipeline) = self.collection_pipeline.take() {
            info!("停止采集管道...");
            pipeline.stop();
            info!("采集管道已停止");
        } else {
            debug!("采集管道未运行，无需停止");
        }
    }

    /// 启动存储管道（后台线程 2）
    pub fn start_storage_pipeline(&mut self) {
        if let Some(pipeline) = &mut self.storage_pipeline {
            info!("启动存储管道（后台线程 2）...");
            if let Err(e) = pipeline.start() {
                error!("存储管道启动失败: {}", e);
                return;
            }
            info!("存储管道启动成功");
        } else {
            warn!("存储管道未初始化，请使用 new_with_storage() 创建管理器");
        }
    }

    /// 停止存储管道
    pub fn stop_storage_pipeline(&mut self) {
        if let Some(pipeline) = &mut self.storage_pipeline {
            info!("停止存储管道...");
            pipeline.stop();
            info!("存储管道已停止");
        } else {
            debug!("存储管道未运行，无需停止");
        }
    }

    pub fn start_process_pipeline(&mut self) {
        if let Some(ref mut pipeline) = self.process_pipeline {
            info!("启动计算管道...");
            pipeline.start();
            info!("计算管道启动成功");
        }
    }

    pub fn stop_process_pipeline(&mut self) {
        if let Some(mut pipeline) = self.process_pipeline.take() {
            info!("停止计算管道...");
            pipeline.stop();
            info!("计算管道已停止");
        }
    }

    /// 启动显示管道（主线程）
    pub fn start_display_pipeline(&mut self) {
        if self.display_pipeline.is_some() {
            warn!("显示管道已经启动，跳过重复启动");
            return;
        }

        info!("启动显示管道（主线程）...");

        // 从配置文件加载显示配置
        let pipeline_config = crate::config::pipeline_config::PipelineConfig::load();
        let config = DisplayPipelineConfig::from_display_config(&pipeline_config.display);

        info!(
            "Display pipeline config: interval={}ms, pipeline_size={}, batch_size={}",
            config.interval.as_millis(),
            config.pipeline_size,
            config.batch_size
        );

        // 创建显示管道
        let mut pipeline = DisplayPipeline::new(config, Arc::clone(&self.shared_buffer));

        // 启动管道
        pipeline.start();

        self.display_pipeline = Some(pipeline);

        info!(
            "显示管道启动成功 - 间隔: {}ms, 管道大小: {}",
            pipeline_config.display.interval_ms, pipeline_config.display.pipeline_size
        );
    }

    /// 停止显示管道
    pub fn stop_display_pipeline(&mut self) {
        if let Some(mut pipeline) = self.display_pipeline.take() {
            info!("停止显示管道...");
            pipeline.stop();
            info!("显示管道已停止");
        } else {
            debug!("显示管道未运行，无需停止");
        }
    }

    /// 启动所有管道
    pub fn start_all(&mut self) {
        info!("启动所有管道...");

        // Start sensor storage pipeline first
        self.start_sensor_storage_pipeline();

        self.start_storage_pipeline();

        if let Some(ref mut pp) = self.process_pipeline {
            if let Some(ref sp) = self.storage_pipeline {
                let last_seq = sp.last_stored_sequence();
                if last_seq > 0 {
                    pp.set_initial_sequence(last_seq);
                    info!("Process pipeline sequence initialized to {}", last_seq);
                }
            }
        }
        self.start_process_pipeline();
        self.start_collection_pipeline();
        self.start_display_pipeline();
        info!("所有管道已启动");
    }

    /// 启动传感器存储管道
    pub fn start_sensor_storage_pipeline(&mut self) {
        if let Some(pipeline) = &mut self.sensor_storage_pipeline {
            info!("启动传感器存储管道...");
            if let Err(e) = pipeline.start() {
                error!("传感器存储管道启动失败: {}", e);
                return;
            }
            info!("传感器存储管道启动成功");
        }
    }

    /// 停止传感器存储管道
    pub fn stop_sensor_storage_pipeline(&mut self) {
        if let Some(pipeline) = &mut self.sensor_storage_pipeline {
            info!("停止传感器存储管道...");
            pipeline.stop();
            info!("传感器存储管道已停止");
        }
    }

    /// 停止所有管道
    pub fn stop_all(&mut self) {
        info!("停止所有管道...");

        // Stop sensor storage pipeline first
        self.stop_sensor_storage_pipeline();

        self.stop_collection_pipeline();
        self.stop_process_pipeline();

        // 再停止存储管道
        self.stop_storage_pipeline();

        // 停止显示管道
        self.stop_display_pipeline();

        info!("所有管道已停止");
    }

    /// 获取共享缓冲区（用于调试和显示管道）
    pub fn get_shared_buffer(&self) -> SharedBuffer {
        Arc::clone(&self.shared_buffer)
    }

    /// 获取共享传感器数据缓冲区
    pub fn get_shared_sensor_buffer(&self) -> Option<SharedSensorBuffer> {
        self.shared_sensor_buffer.clone()
    }

    /// 获取显示管道的可变引用
    ///
    /// 用于在主线程中通过 QTimer 驱动显示更新
    pub fn get_display_pipeline_mut(&mut self) -> Option<&mut DisplayPipeline> {
        self.display_pipeline.as_mut()
    }

    /// 检查显示管道是否运行中
    pub fn is_display_running(&self) -> bool {
        self.display_pipeline.is_some()
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
        let len = self.storage_pipeline.as_ref().map(|p| p.queue_len());
        if let Some(l) = len {
            trace!("存储队列长度: {}", l);
        }
        len
    }

    /// 获取最后存储的序列号
    pub fn get_last_stored_sequence(&self) -> Option<u64> {
        let seq = self
            .storage_pipeline
            .as_ref()
            .map(|p| p.last_stored_sequence());
        if let Some(s) = seq {
            trace!("最后存储的序列号: {}", s);
        }
        seq
    }

    pub fn get_config_provider(&self) -> Option<Arc<ConfigProvider>> {
        self.config_provider.clone()
    }

    pub fn get_calibration_service(&self) -> Option<&CalibrationService> {
        self.calibration_service.as_ref()
    }

    pub fn get_event_bus(&self) -> Option<&EventBus> {
        self.event_bus.as_ref()
    }
}

impl Drop for PipelineManager {
    fn drop(&mut self) {
        info!("PipelineManager 正在销毁，停止所有管道");
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

        // 测试输出
        println!("[TEST] Collection stats after 1 second:");
        println!("  - Total collections: {}", stats.total_collections);
        println!("  - Success count: {}", stats.success_count);
        println!("  - Error count: {}", stats.error_count);

        // 应该至少采集了 8 次（考虑启动延迟）
        assert!(
            stats.total_collections >= 8,
            "Should collect at least 8 times in 1 second at 10Hz, got {}",
            stats.total_collections
        );

        // 成功率应该很高
        assert!(
            stats.success_count >= 8,
            "Should have at least 8 successful collections"
        );

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
        // 测试输出，使用 println! 而不是 tracing
        println!("[TEST] Processed data:");
        println!("  - Sequence: {}", data.sequence_number);
        println!("  - Load: {:.2} tons", data.current_load);
        println!("  - Radius: {:.2} m", data.working_radius);
        println!("  - Angle: {:.2}°", data.boom_angle);
        println!("  - Moment %: {:.2}%", data.moment_percentage);
        println!("  - Is danger: {}", data.is_danger);

        // 验证数据合理性
        assert!(data.current_load >= 0.0, "Load should be non-negative");
        assert!(data.working_radius >= 0.0, "Radius should be non-negative");
        assert!(
            data.boom_angle >= 0.0 && data.boom_angle <= 90.0,
            "Angle should be between 0 and 90 degrees"
        );
        assert!(
            data.moment_percentage >= 0.0,
            "Moment percentage should be non-negative"
        );

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

        println!("[TEST] History buffer contains {} records", history.len());

        // 应该有多条历史记录
        assert!(
            history.len() >= 3,
            "Should have at least 3 historical records"
        );

        // 验证序列号递增
        for i in 0..history.len().saturating_sub(1) {
            assert!(
                history[i].sequence_number > history[i + 1].sequence_number,
                "History should be in reverse chronological order"
            );
        }

        manager.stop_collection_pipeline();
    }
}
