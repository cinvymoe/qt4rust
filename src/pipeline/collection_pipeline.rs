// 采集管道（异步版本）

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, RwLock};
use std::time::Duration;
use tokio::task::JoinHandle;
use crate::repositories::CraneDataRepository;
use crate::models::{ProcessedData, SensorData};
use crate::models::sensor_calibration::{SensorCalibration, AlarmThresholds};
use crate::models::rated_load_table::RatedLoadTable;
use crate::models::crane_config::CraneConfig;
use super::shared_buffer::SharedBuffer;
use super::event_channel::StorageEventSender;
use super::sensor_data_event_channel::SensorDataEventSender;
use super::filter_buffer::FilterBuffer;
use super::shared_sensor_buffer::SharedSensorBuffer;

/// 采集错误类型
#[derive(Debug)]
enum CollectionError {
    /// 采集失败
    Collection(String),
    /// 任务 panic
    Panic(String),
    /// 任务取消
    Cancelled(String),
}

/// 管道上下文（包含所有共享状态）
struct PipelineContext {
    config: CollectionPipelineConfig,
    repository: Arc<CraneDataRepository>,
    display_buffer: SharedBuffer,
    filter_buffer: Option<Arc<Mutex<FilterBuffer>>>,
    running: Arc<AtomicBool>,
    sequence_number: Arc<AtomicU64>,
    alarm_callback: Option<Arc<dyn Fn(ProcessedData) + Send + Sync>>,
    danger_cleared_callback: Option<Arc<dyn Fn() + Send + Sync>>,
    storage_event_sender: Option<StorageEventSender>,
    sensor_storage_sender: Option<SensorDataEventSender>,
    shared_sensor_buffer: Option<SharedSensorBuffer>,
    sensor_calibration: Option<Arc<RwLock<SensorCalibration>>>,
    rated_load_table: Option<Arc<RwLock<RatedLoadTable>>>,
    alarm_thresholds: Option<Arc<RwLock<AlarmThresholds>>>,
}

/// 采集循环状态
struct CollectionLoopState {
    consecutive_failures: u32,
    interval_timer: tokio::time::Interval,
    previous_danger: bool,
}

impl CollectionLoopState {
    fn new(interval: Duration) -> Self {
        Self {
            consecutive_failures: 0,
            interval_timer: tokio::time::interval(interval),
            previous_danger: false,
        }
    }
}

/// 采集管道配置
#[derive(Debug, Clone)]
pub struct CollectionPipelineConfig {
    /// 采集间隔
    pub interval: Duration,
    
    /// 失败重试次数
    pub max_retries: u32,
    
    /// 重试延迟
    pub retry_delay: Duration,
    
    /// 断连检测阈值（连续失败次数）
    pub disconnect_threshold: u32,
    
    /// 是否启用 panic 恢复
    pub enable_panic_recovery: bool,
    
    /// 最大重启次数
    pub max_restarts: usize,
}

impl Default for CollectionPipelineConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_millis(100),
            max_retries: 3,
            retry_delay: Duration::from_millis(10),
            disconnect_threshold: 10,
            enable_panic_recovery: true,
            max_restarts: 5,
        }
    }
}

/// 采集管道（异步版本）
pub struct CollectionPipeline {
    config: CollectionPipelineConfig,
    repository: Arc<CraneDataRepository>,
    display_buffer: SharedBuffer,
    filter_buffer: Option<Arc<Mutex<FilterBuffer>>>,
    running: Arc<AtomicBool>,
    sequence_number: Arc<AtomicU64>,
    handle: Option<JoinHandle<()>>,
    alarm_callback: Option<Arc<dyn Fn(ProcessedData) + Send + Sync>>,
    danger_cleared_callback: Option<Arc<dyn Fn() + Send + Sync>>,
    storage_event_sender: Option<StorageEventSender>,
    sensor_storage_sender: Option<SensorDataEventSender>,
    shared_sensor_buffer: Option<SharedSensorBuffer>,
    // 热重载配置引用
    sensor_calibration: Option<Arc<RwLock<SensorCalibration>>>,
    rated_load_table: Option<Arc<RwLock<RatedLoadTable>>>,
    alarm_thresholds: Option<Arc<RwLock<AlarmThresholds>>>,
}

impl CollectionPipeline {
    /// 创建新的采集管道
    pub fn new(
        config: CollectionPipelineConfig,
        repository: Arc<CraneDataRepository>,
        display_buffer: SharedBuffer,
    ) -> Self {
        Self {
            config,
            repository,
            display_buffer,
            filter_buffer: None,
            running: Arc::new(AtomicBool::new(false)),
            sequence_number: Arc::new(AtomicU64::new(0)),
            handle: None,
            alarm_callback: None,
            danger_cleared_callback: None,
            storage_event_sender: None,
            sensor_storage_sender: None,
            shared_sensor_buffer: None,
            sensor_calibration: None,
            rated_load_table: None,
            alarm_thresholds: None,
        }
    }

    /// 创建采集管道并附带事件发送器
    pub fn with_event_sender(
        config: CollectionPipelineConfig,
        repository: Arc<CraneDataRepository>,
        display_buffer: SharedBuffer,
        storage_event_sender: StorageEventSender,
    ) -> Self {
        Self {
            config,
            repository,
            display_buffer,
            filter_buffer: None,
            running: Arc::new(AtomicBool::new(false)),
            sequence_number: Arc::new(AtomicU64::new(0)),
            handle: None,
            alarm_callback: None,
            danger_cleared_callback: None,
            storage_event_sender: Some(storage_event_sender),
            sensor_storage_sender: None,
            shared_sensor_buffer: None,
            sensor_calibration: None,
            rated_load_table: None,
            alarm_thresholds: None,
        }
    }

    /// 创建采集管道并附带存储发送器和传感器存储发送器
    pub fn with_storage_sender(
        config: CollectionPipelineConfig,
        repository: Arc<CraneDataRepository>,
        display_buffer: SharedBuffer,
        storage_event_sender: StorageEventSender,
        sensor_storage_sender: SensorDataEventSender,
        shared_sensor_buffer: SharedSensorBuffer,
    ) -> Self {
        Self {
            config,
            repository,
            display_buffer,
            filter_buffer: None,
            running: Arc::new(AtomicBool::new(false)),
            sequence_number: Arc::new(AtomicU64::new(0)),
            handle: None,
            alarm_callback: None,
            danger_cleared_callback: None,
            storage_event_sender: Some(storage_event_sender),
            sensor_storage_sender: Some(sensor_storage_sender),
            shared_sensor_buffer: Some(shared_sensor_buffer),
            sensor_calibration: None,
            rated_load_table: None,
            alarm_thresholds: None,
        }
    }

    /// 创建采集管道（写入滤波缓冲区，用于多速率架构）
    pub fn with_filter_buffer(
        config: CollectionPipelineConfig,
        repository: Arc<CraneDataRepository>,
        filter_buffer: Arc<Mutex<FilterBuffer>>,
    ) -> Self {
        Self {
            config,
            repository,
            display_buffer: Arc::new(std::sync::RwLock::new(
                super::shared_buffer::ProcessedDataBuffer::new(100)
            )),
            filter_buffer: Some(filter_buffer),
            running: Arc::new(AtomicBool::new(false)),
            sequence_number: Arc::new(AtomicU64::new(0)),
            handle: None,
            alarm_callback: None,
            danger_cleared_callback: None,
            storage_event_sender: None,
            sensor_storage_sender: None,
            shared_sensor_buffer: None,
            sensor_calibration: None,
            rated_load_table: None,
            alarm_thresholds: None,
        }
    }
    
    /// 设置热重载配置引用
    pub fn set_hot_reload_config(
        &mut self,
        sensor_calibration: Arc<RwLock<SensorCalibration>>,
        rated_load_table: Arc<RwLock<RatedLoadTable>>,
        alarm_thresholds: Arc<RwLock<AlarmThresholds>>,
    ) {
        self.sensor_calibration = Some(sensor_calibration.clone());
        self.rated_load_table = Some(rated_load_table.clone());
        self.alarm_thresholds = Some(alarm_thresholds.clone());
        
        if let Ok(cal) = sensor_calibration.read() {
            tracing::info!("🔧 [CollectionPipeline] 热重载配置已设置");
            tracing::info!("📋 [初始标定参数] weight: zero_ad={:.2}, zero_value={:.2}, scale_ad={:.2}, scale_value={:.2}, multiplier={:.2}",
                cal.weight.zero_ad,
                cal.weight.zero_value,
                cal.weight.scale_ad,
                cal.weight.scale_value,
                cal.weight.multiplier);
        }
        
        if let Ok(thresholds) = alarm_thresholds.read() {
            tracing::info!("⚠️  [CollectionPipeline] 预警阈值已设置: warning={}%, alarm={}%",
                thresholds.moment.warning_percentage,
                thresholds.moment.alarm_percentage);
        }
    }
    
    /// 设置报警回调
    /// 
    /// 当检测到报警状态时，会调用此回调函数
    pub fn set_alarm_callback<F>(&mut self, callback: F)
    where
        F: Fn(ProcessedData) + Send + Sync + 'static,
    {
        self.alarm_callback = Some(Arc::new(callback));
    }
    
    /// 设置报警解除回调
    /// 
    /// 当报警状态解除时（is_danger 从 true 变为 false），会调用此回调函数
    pub fn set_danger_cleared_callback<F>(&mut self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.danger_cleared_callback = Some(Arc::new(callback));
    }
    
    /// 设置初始序列号
    /// 
    /// 用于在程序重启后继续之前的序列号
    pub fn set_initial_sequence(&mut self, sequence: u64) {
        self.sequence_number.store(sequence, Ordering::Relaxed);
        tracing::info!(" Collection pipeline sequence initialized to {}", sequence);
    }
    
    pub fn start(&mut self) {
        if self.running.load(Ordering::Relaxed) {
            tracing::warn!(" Collection pipeline already running");
            return;
        }
        
        self.running.store(true, Ordering::Relaxed);

        let context = self.create_pipeline_context();
        let handle = qt_threading_utils::runtime::global_runtime().spawn(async move {
            Self::run_collection_loop(context).await;
        });
        
        self.handle = Some(handle);
    }
    
    /// 创建管道上下文（克隆所有需要的共享状态）
    fn create_pipeline_context(&self) -> PipelineContext {
        PipelineContext {
            config: self.config.clone(),
            repository: Arc::clone(&self.repository),
            display_buffer: Arc::clone(&self.display_buffer),
            filter_buffer: self.filter_buffer.clone(),
            running: Arc::clone(&self.running),
            sequence_number: Arc::clone(&self.sequence_number),
            alarm_callback: self.alarm_callback.clone(),
            danger_cleared_callback: self.danger_cleared_callback.clone(),
            storage_event_sender: self.storage_event_sender.clone(),
            sensor_storage_sender: self.sensor_storage_sender.clone(),
            shared_sensor_buffer: self.shared_sensor_buffer.clone(),
            sensor_calibration: self.sensor_calibration.clone(),
            rated_load_table: self.rated_load_table.clone(),
            alarm_thresholds: self.alarm_thresholds.clone(),
        }
    }
    
    /// 运行采集循环
    async fn run_collection_loop(ctx: PipelineContext) {
        tracing::info!(" Collection pipeline started (mode: {})", 
            if ctx.filter_buffer.is_some() { "filter" } else { "legacy" });
        
        let mut state = CollectionLoopState::new(ctx.config.interval);

        while ctx.running.load(Ordering::Relaxed) {
            state.interval_timer.tick().await;

            let result = Self::collect_sensor_data(&ctx.repository, &ctx.config).await;
            
            match result {
                Ok(sensor_data) => {
                    state.consecutive_failures = 0;
                    Self::handle_sensor_data_success(sensor_data, &ctx, &mut state).await;
                }
                Err(e) => {
                    Self::handle_sensor_data_error(e, &ctx, &mut state);
                }
            }
        }
        
        tracing::info!(" Collection pipeline stopped");
    }
    
    /// 采集传感器数据
    async fn collect_sensor_data(
        repository: &Arc<CraneDataRepository>,
        config: &CollectionPipelineConfig,
    ) -> Result<SensorData, CollectionError> {
        let repo = Arc::clone(repository);
        let cfg = config.clone();
        
        tokio::task::spawn_blocking(move || {
            Self::collect_with_retry(&repo, &cfg)
        })
        .await
        .map_err(|e| {
            if e.is_panic() {
                CollectionError::Panic(format!("{:?}", e))
            } else {
                CollectionError::Cancelled(e.to_string())
            }
        })?
        .map_err(CollectionError::Collection)
    }
    
    /// 处理传感器数据采集成功
    async fn handle_sensor_data_success(
        sensor_data: SensorData,
        ctx: &PipelineContext,
        state: &mut CollectionLoopState,
    ) {
        // 写入共享传感器缓冲区（供校准界面使用）
        Self::write_to_shared_sensor_buffer(&sensor_data, &ctx.shared_sensor_buffer);

        if let Some(ref fb) = ctx.filter_buffer {
            // 多速率模式: 只写入滤波缓冲区
            Self::handle_filter_mode(sensor_data, fb);
        } else {
            // 遗留模式: 处理数据并写入显示缓冲区和存储
            Self::handle_legacy_mode(sensor_data, ctx, state);
        }
    }
    
    /// 写入共享传感器缓冲区
    fn write_to_shared_sensor_buffer(
        sensor_data: &SensorData,
        shared_sensor_buffer: &Option<SharedSensorBuffer>,
    ) {
        if let Some(ref buffer) = shared_sensor_buffer {
            if let Ok(mut buf) = buffer.write() {
                buf.push(sensor_data.clone());
            }
        }
    }
    
    /// 处理滤波模式
    fn handle_filter_mode(sensor_data: SensorData, filter_buffer: &Arc<Mutex<FilterBuffer>>) {
        if let Ok(mut fb_guard) = filter_buffer.lock() {
            tracing::debug!("[CollectionPipeline] 写入FilterBuffer: ad1={:.2}, ad2={:.2}, ad3={:.2}",
                sensor_data.ad1_load, sensor_data.ad2_radius, sensor_data.ad3_angle);
            fb_guard.push(sensor_data);
        }
    }
    
    /// 处理遗留模式
    fn handle_legacy_mode(
        sensor_data: SensorData,
        ctx: &PipelineContext,
        state: &mut CollectionLoopState,
    ) {
        let seq = ctx.sequence_number.fetch_add(1, Ordering::Relaxed);
        
        // 处理数据（AD转换）
        let processed = Self::process_sensor_data(
            &sensor_data,
            &ctx.repository,
            &ctx.sensor_calibration,
            &ctx.rated_load_table,
            &ctx.alarm_thresholds,
            seq,
        );

        // 发送到存储管道
        Self::send_to_storage(&sensor_data, &processed, ctx);

        // 检测报警状态变化
        Self::check_alarm_state(&processed, &ctx.alarm_callback, &ctx.danger_cleared_callback, state);

        // 写入显示缓冲区
        Self::write_to_display_buffer(&processed, &ctx.display_buffer);
    }
    
    /// 处理传感器数据（AD转换）
    fn process_sensor_data(
        sensor_data: &SensorData,
        repository: &Arc<CraneDataRepository>,
        sensor_calibration: &Option<Arc<RwLock<SensorCalibration>>>,
        rated_load_table: &Option<Arc<RwLock<RatedLoadTable>>>,
        alarm_thresholds: &Option<Arc<RwLock<AlarmThresholds>>>,
        seq: u64,
    ) -> ProcessedData {
        if let (Some(cal), Some(table), Some(thresholds)) = (sensor_calibration, rated_load_table, alarm_thresholds) {
            Self::process_with_hot_reload(sensor_data, repository, cal, table, thresholds, seq)
        } else {
            Self::process_with_static_config(sensor_data, repository, seq)
        }
    }
    
    /// 使用热重载配置处理数据
    fn process_with_hot_reload(
        sensor_data: &SensorData,
        _repository: &Arc<CraneDataRepository>,
        sensor_calibration: &Arc<RwLock<SensorCalibration>>,
        rated_load_table: &Arc<RwLock<RatedLoadTable>>,
        alarm_thresholds: &Arc<RwLock<AlarmThresholds>>,
        seq: u64,
    ) -> ProcessedData {
        let cal_guard = sensor_calibration.read().unwrap();
        let table_guard = rated_load_table.read().unwrap();
        let thresholds_guard = alarm_thresholds.read().unwrap();
        
        tracing::info!("🔥 [CollectionPipeline] 使用热重载配置进行AD转换");
        tracing::info!("📊 [标定参数] weight: zero_ad={:.2}, zero_value={:.2}, scale_ad={:.2}, scale_value={:.2}, multiplier={:.2}",
            cal_guard.weight.zero_ad,
            cal_guard.weight.zero_value,
            cal_guard.weight.scale_ad,
            cal_guard.weight.scale_value,
            cal_guard.weight.multiplier);
        
        tracing::info!("⚠️  [预警阈值] warning={}%, alarm={}%",
            thresholds_guard.moment.warning_percentage,
            thresholds_guard.moment.alarm_percentage);
        
        let hot_config = CraneConfig {
            sensor_calibration: cal_guard.clone(),
            rated_load_table: table_guard.clone(),
            alarm_thresholds: thresholds_guard.clone(),
        };
        
        let processed = ProcessedData::from_sensor_data_with_config(
            sensor_data.clone(), 
            &hot_config, 
            seq,
        );
        tracing::info!("✅ [CollectionPipeline] AD转换完成: ad1={:.2} -> load={:.2}吨",
            sensor_data.ad1_load, processed.current_load);
        
        processed
    }
    
    /// 使用静态配置处理数据
    fn process_with_static_config(
        sensor_data: &SensorData,
        repository: &Arc<CraneDataRepository>,
        seq: u64,
    ) -> ProcessedData {
        tracing::warn!("⚠️  [CollectionPipeline] 热重载配置未设置，使用静态配置");
        
        match repository.get_config() {
            Ok(config) => {
                let processed = ProcessedData::from_sensor_data_with_config(
                    sensor_data.clone(), 
                    &config, 
                    seq,
                );
                tracing::debug!("[CollectionPipeline] 静态配置AD转换: ad1={:.2} -> load={:.2}吨",
                    sensor_data.ad1_load, processed.current_load);
                processed
            }
            Err(e) => {
                tracing::warn!("[CollectionPipeline] 无法加载配置，使用简单转换: {}", e);
                ProcessedData::from_sensor_data(sensor_data.clone(), seq)
            }
        }
    }
    
    /// 发送数据到存储管道
    fn send_to_storage(
        sensor_data: &SensorData,
        processed: &ProcessedData,
        ctx: &PipelineContext,
    ) {
        if let Some(ref sender) = ctx.sensor_storage_sender {
            if let Err(e) = sender.try_send_data(vec![sensor_data.clone()]) {
                tracing::warn!("Failed to send sensor data to storage: {}", e);
            }
        }

        if let Some(ref sender) = ctx.storage_event_sender {
            if let Err(e) = sender.try_send_data(vec![processed.clone()]) {
                tracing::warn!("Failed to send data to storage: {}", e);
            }
        }
    }
    
    /// 检测报警状态变化
    fn check_alarm_state(
        processed: &ProcessedData,
        alarm_callback: &Option<Arc<dyn Fn(ProcessedData) + Send + Sync>>,
        danger_cleared_callback: &Option<Arc<dyn Fn() + Send + Sync>>,
        state: &mut CollectionLoopState,
    ) {
        let current_danger = processed.is_danger;

        if current_danger && !state.previous_danger {
            tracing::warn!("[ALARM] Danger detected! Moment: {:.1}%", processed.moment_percentage);
            if let Some(ref callback) = alarm_callback {
                callback(processed.clone());
            }
        } else if !current_danger && state.previous_danger {
            tracing::info!("[ALARM] Danger cleared");
            if let Some(ref callback) = danger_cleared_callback {
                callback();
            }
        }

        state.previous_danger = current_danger;
    }
    
    /// 写入显示缓冲区
    fn write_to_display_buffer(processed: &ProcessedData, display_buffer: &SharedBuffer) {
        match display_buffer.try_write() {
            Ok(mut buf) => { buf.push(processed.clone()); }
            Err(_) => { tracing::warn!(" Failed to acquire buffer lock, skipping write"); }
        }
    }
    
    /// 处理传感器数据采集错误
    fn handle_sensor_data_error(
        error: CollectionError,
        ctx: &PipelineContext,
        state: &mut CollectionLoopState,
    ) {
        match error {
            CollectionError::Collection(e) => {
                state.consecutive_failures += 1;
                tracing::error!(" Collection failed: {} (consecutive: {})", e, state.consecutive_failures);
                
                if ctx.filter_buffer.is_none() {
                    if let Ok(mut buf) = ctx.display_buffer.try_write() {
                        buf.record_error();
                    }
                }
                
                if state.consecutive_failures >= ctx.config.disconnect_threshold {
                    tracing::error!(" Sensor disconnected (threshold reached)");
                }
            }
            CollectionError::Panic(e) => {
                tracing::error!("[PANIC] Collection task panicked: {}", e);
            }
            CollectionError::Cancelled(e) => {
                tracing::error!(" Collection task cancelled: {}", e);
            }
        }
    }
    
    /// 停止管道
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        
        if let Some(handle) = self.handle.take() {
            qt_threading_utils::runtime::global_runtime().block_on(async {
                let _ = handle.await;
            });
        }
    }
    
    /// 带重试的数据采集
    fn collect_with_retry(
        repository: &CraneDataRepository,
        config: &CollectionPipelineConfig,
    ) -> Result<SensorData, String> {
        let mut last_error = String::new();
        
        for attempt in 0..=config.max_retries {
            match repository.get_latest_sensor_data() {
                Ok(data) => return Ok(data),
                Err(e) => {
                    last_error = e;
                    if attempt < config.max_retries {
                        std::thread::sleep(config.retry_delay);
                    }
                }
            }
        }
        
        Err(format!("Failed after {} retries: {}", config.max_retries, last_error))
    }
}

impl Drop for CollectionPipeline {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::RwLock;
    
    #[test]
    fn test_config_default() {
        let config = CollectionPipelineConfig::default();
        
        assert_eq!(config.interval, Duration::from_millis(100));
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.retry_delay, Duration::from_millis(10));
        assert_eq!(config.disconnect_threshold, 10);
        assert!(config.enable_panic_recovery);
        assert_eq!(config.max_restarts, 5);
    }
    
    #[test]
    fn test_pipeline_creation() {
        use crate::config::config_manager::ConfigManager;
        
        let config = CollectionPipelineConfig::default();
        let config_manager = Arc::new(ConfigManager::default());
        let repository = Arc::new(CraneDataRepository::new(config_manager));
        let buffer = Arc::new(RwLock::new(
            super::super::shared_buffer::ProcessedDataBuffer::new(100)
        ));
        
        let pipeline = CollectionPipeline::new(config, repository, buffer);
        
        assert!(!pipeline.running.load(Ordering::Relaxed));
    }
}
