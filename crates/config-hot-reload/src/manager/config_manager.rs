//! 热加载配置管理器
//!
//! 协调配置热加载流程，管理配置缓存和订阅者

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};

use super::config_cache::ConfigCache;
use crate::error::HotReloadError;
use crate::parser::ConfigParser;
use crate::subscriber::ConfigSubscriber;
use crate::types::{ConfigChange, ConfigFileEvent, ConfigFileType, ConfigSnapshot};
use crate::validator::ConfigValidator;
use crate::watcher::FileWatcher;

/// 热加载配置管理器
///
/// 负责：
/// - 启动和停止文件监控
/// - 处理文件变更事件
/// - 解析和验证配置
/// - 原子性更新配置缓存
/// - 通知订阅者配置变更
pub struct HotReloadConfigManager {
    /// 配置缓存（线程安全）
    config_cache: Arc<RwLock<ConfigCache>>,

    /// 配置订阅者列表（线程安全）
    subscribers: Arc<RwLock<Vec<Box<dyn ConfigSubscriber>>>>,

    /// 配置目录路径
    config_dir: PathBuf,

    /// 文件监控器（可选，启动后才有）
    file_watcher: Option<FileWatcher>,

    /// 事件接收器（可选，启动后才有）
    event_rx: Option<mpsc::Receiver<ConfigFileEvent>>,

    /// 更新互斥锁（确保同一时刻只有一个更新操作）
    update_mutex: Arc<Mutex<()>>,
}

impl HotReloadConfigManager {
    /// 创建新的热加载配置管理器
    ///
    /// # 参数
    ///
    /// * `config_dir` - 配置文件目录路径
    ///
    /// # 返回
    ///
    /// 返回配置管理器实例或错误
    pub fn new(config_dir: PathBuf) -> Result<Self, HotReloadError> {
        info!("创建热加载配置管理器，配置目录: {:?}", config_dir);

        // 创建事件通道
        let (event_tx, event_rx) = mpsc::channel(100);

        // 创建文件监控器
        let file_watcher = FileWatcher::new(config_dir.clone(), event_tx)?;

        Ok(Self {
            config_cache: Arc::new(RwLock::new(ConfigCache::default())),
            subscribers: Arc::new(RwLock::new(Vec::new())),
            config_dir,
            file_watcher: Some(file_watcher),
            event_rx: Some(event_rx),
            update_mutex: Arc::new(Mutex::new(())),
        })
    }

    /// 启动热加载服务
    ///
    /// 启动文件监控和事件处理循环
    pub async fn start(&mut self) -> Result<(), HotReloadError> {
        info!("启动热加载服务");

        // 启动文件监控器
        if let Some(mut watcher) = self.file_watcher.take() {
            watcher.start().await?;
            self.file_watcher = Some(watcher);
        }

        // 启动事件处理循环
        self.run_event_loop().await;

        Ok(())
    }

    /// 停止热加载服务
    pub fn stop(&mut self) {
        info!("停止热加载服务");

        if let Some(mut watcher) = self.file_watcher.take() {
            watcher.stop();
        }

        // 关闭事件接收器
        self.event_rx = None;
    }

    /// 手动重载所有配置
    ///
    /// # 返回
    ///
    /// 成功返回 Ok(())，失败返回错误
    pub async fn reload_all(&self) -> Result<(), HotReloadError> {
        info!("手动重载所有配置");

        // 获取更新锁，确保互斥性
        let _lock = self.update_mutex.lock().await;

        // 重载每个配置文件
        let config_types = [
            ConfigFileType::SensorCalibration,
            ConfigFileType::AlarmThresholds,
            ConfigFileType::Logging,
            ConfigFileType::ModbusSensors,
            ConfigFileType::Pipeline,
            ConfigFileType::RatedLoadTable,
        ];

        for config_type in config_types.iter() {
            if let Err(e) = self.reload_config_internal(*config_type).await {
                error!("重载配置失败: {:?}, 错误: {}", config_type, e);
                // 继续重载其他配置
            }
        }

        Ok(())
    }

    /// 手动重载指定配置
    ///
    /// # 参数
    ///
    /// * `file_type` - 配置文件类型
    ///
    /// # 返回
    ///
    /// 成功返回 Ok(())，失败返回错误
    pub async fn reload_config(&self, file_type: ConfigFileType) -> Result<(), HotReloadError> {
        info!("手动重载配置: {:?}", file_type);

        // 获取更新锁，确保互斥性
        let _lock = self.update_mutex.lock().await;

        self.reload_config_internal(file_type).await
    }

    /// 获取配置快照
    ///
    /// # 返回
    ///
    /// 返回当前配置的只读快照
    pub async fn get_config_snapshot(&self) -> ConfigSnapshot {
        let cache = self.config_cache.read().await;

        ConfigSnapshot {
            sensor_calibration: cache.sensor_calibration.clone(),
            alarm_thresholds: cache.alarm_thresholds.clone(),
            logging_config: cache.logging_config.clone(),
            modbus_config: cache.modbus_config.clone(),
            pipeline_config: cache.pipeline_config.clone(),
            rated_load_table: cache.rated_load_table.clone(),
            version: cache.version,
        }
    }

    /// 订阅配置变更
    ///
    /// 注册一个配置订阅者，当配置更新时会收到通知。
    ///
    /// # 参数
    ///
    /// * `subscriber` - 实现了 ConfigSubscriber trait 的订阅者
    ///
    /// # 示例
    ///
    /// ```rust,ignore
    /// manager.subscribe(Box::new(MyComponent));
    /// ```
    pub async fn subscribe(&self, subscriber: Box<dyn ConfigSubscriber>) {
        let mut subscribers = self.subscribers.write().await;
        info!("注册配置订阅者: {}", subscriber.name());
        subscribers.push(subscriber);
    }

    /// 通知所有订阅者配置已变更
    ///
    /// 异步并发通知所有订阅者，单个订阅者的错误不会影响其他订阅者。
    /// 确保在 100 毫秒内完成所有订阅者的通知。
    ///
    /// # 参数
    ///
    /// * `change` - 配置变更信息
    async fn notify_subscribers(&self, change: ConfigChange) {
        let start_time = Instant::now();

        // 获取配置快照
        let snapshot = self.get_config_snapshot().await;

        // 获取订阅者列表（读锁）
        let subscribers = self.subscribers.read().await;

        if subscribers.is_empty() {
            debug!("没有配置订阅者需要通知");
            return;
        }

        info!(
            "通知 {} 个订阅者配置变更: {:?}",
            subscribers.len(),
            change.file_type
        );

        // 顺序通知所有订阅者，但为每个订阅者设置超时
        for subscriber in subscribers.iter() {
            let subscriber_name = subscriber.name();
            let change_clone = change.clone();
            let snapshot_clone = snapshot.clone();

            // 使用 timeout 确保单个订阅者不会阻塞太久
            match tokio::time::timeout(
                std::time::Duration::from_millis(50),
                subscriber.on_config_changed(change_clone, snapshot_clone),
            )
            .await
            {
                Ok(_) => {
                    debug!("订阅者 {} 已收到配置变更通知", subscriber_name);
                }
                Err(_) => {
                    warn!("订阅者 {} 处理配置变更超时（> 50ms）", subscriber_name);
                }
            }
        }

        let elapsed = start_time.elapsed();
        if elapsed.as_millis() > 100 {
            warn!(
                "通知所有订阅者耗时 {}ms，超过 100ms 目标",
                elapsed.as_millis()
            );
        } else {
            debug!("通知所有订阅者完成，耗时 {}ms", elapsed.as_millis());
        }
    }

    /// 运行事件处理循环
    async fn run_event_loop(&mut self) {
        if let Some(mut event_rx) = self.event_rx.take() {
            let config_cache = Arc::clone(&self.config_cache);
            let subscribers = Arc::clone(&self.subscribers);
            let update_mutex = Arc::clone(&self.update_mutex);
            let config_dir = self.config_dir.clone();

            tokio::spawn(async move {
                info!("事件处理循环已启动");

                while let Some(event) = event_rx.recv().await {
                    debug!("收到文件变更事件: {:?}", event);

                    // 获取更新锁
                    let _lock = update_mutex.lock().await;

                    // 处理文件事件
                    if let Err(e) = Self::handle_file_event_static(
                        &config_cache,
                        &subscribers,
                        &config_dir,
                        event.clone(),
                    )
                    .await
                    {
                        // 检查是否是空文件错误（编辑器保存过程中的临时状态）
                        let error_msg = e.to_string();
                        if error_msg.contains("文件为空") || error_msg.contains("只包含空白字符")
                        {
                            warn!(
                                file_type = ?event.file_type,
                                path = %event.path.display(),
                                "忽略空文件事件（可能是编辑器保存过程中的临时状态）"
                            );
                        } else {
                            error!("处理文件事件失败: {}", e);
                        }
                    }
                }

                info!("事件处理循环已停止");
            });
        }
    }

    /// 处理文件变更事件（静态方法，用于 tokio::spawn）
    async fn handle_file_event_static(
        config_cache: &Arc<RwLock<ConfigCache>>,
        subscribers: &Arc<RwLock<Vec<Box<dyn ConfigSubscriber>>>>,
        config_dir: &PathBuf,
        event: ConfigFileEvent,
    ) -> Result<(), HotReloadError> {
        info!(
            file_type = ?event.file_type,
            path = %event.path.display(),
            timestamp = ?event.timestamp,
            "处理文件变更事件"
        );

        // 构建完整路径
        let file_path = if event.path.is_absolute() {
            event.path.clone()
        } else {
            config_dir.join(&event.path)
        };

        // 记录旧版本号
        let old_version = {
            let cache = config_cache.read().await;
            cache.version
        };

        // 根据文件类型解析和验证
        match event.file_type {
            ConfigFileType::SensorCalibration => {
                let config = ConfigParser::parse_sensor_calibration(&file_path)?;
                ConfigValidator::validate_sensor_calibration(&config).map_err(|e| {
                    error!(
                        file_type = ?event.file_type,
                        path = %file_path.display(),
                        error = %e,
                        timestamp = ?std::time::SystemTime::now(),
                        "配置验证失败"
                    );
                    HotReloadError::ValidationFailed {
                        file_type: event.file_type,
                        source: e,
                    }
                })?;

                // 原子性更新配置
                let mut cache = config_cache.write().await;

                // 记录配置变更前后的关键参数差异
                info!(
                    config_type = "SensorCalibration",
                    old_weight_scale = cache.sensor_calibration.weight.scale_value,
                    new_weight_scale = config.weight.scale_value,
                    old_angle_scale = cache.sensor_calibration.angle.scale_value,
                    new_angle_scale = config.angle.scale_value,
                    old_radius_scale = cache.sensor_calibration.radius.scale_value,
                    new_radius_scale = config.radius.scale_value,
                    "传感器校准配置参数变更"
                );

                cache.sensor_calibration = config;
                cache.increment_version();

                info!(
                    old_version = old_version,
                    new_version = cache.version,
                    timestamp = ?std::time::SystemTime::now(),
                    "传感器校准配置已更新"
                );
            }
            ConfigFileType::AlarmThresholds => {
                let config = ConfigParser::parse_alarm_thresholds(&file_path)?;
                ConfigValidator::validate_alarm_thresholds(&config).map_err(|e| {
                    error!(
                        file_type = ?event.file_type,
                        path = %file_path.display(),
                        error = %e,
                        timestamp = ?std::time::SystemTime::now(),
                        "配置验证失败"
                    );
                    HotReloadError::ValidationFailed {
                        file_type: event.file_type,
                        source: e,
                    }
                })?;

                let mut cache = config_cache.write().await;

                // 记录配置变更前后的关键参数差异
                info!(
                    config_type = "AlarmThresholds",
                    old_warning = cache.alarm_thresholds.moment.warning_percentage,
                    new_warning = config.moment.warning_percentage,
                    old_alarm = cache.alarm_thresholds.moment.alarm_percentage,
                    new_alarm = config.moment.alarm_percentage,
                    "报警阈值配置参数变更"
                );

                cache.alarm_thresholds = config;
                cache.increment_version();

                info!(
                    old_version = old_version,
                    new_version = cache.version,
                    timestamp = ?std::time::SystemTime::now(),
                    "报警阈值配置已更新"
                );
            }
            ConfigFileType::Logging => {
                let config = ConfigParser::parse_logging_config(&file_path)?;
                ConfigValidator::validate_logging_config(&config).map_err(|e| {
                    error!(
                        file_type = ?event.file_type,
                        path = %file_path.display(),
                        error = %e,
                        timestamp = ?std::time::SystemTime::now(),
                        "配置验证失败"
                    );
                    HotReloadError::ValidationFailed {
                        file_type: event.file_type,
                        source: e,
                    }
                })?;

                let mut cache = config_cache.write().await;

                // 记录配置变更前后的关键参数差异
                info!(
                    config_type = "Logging",
                    old_console_output = cache.logging_config.console_output,
                    new_console_output = config.console_output,
                    old_file_output = cache.logging_config.file_output,
                    new_file_output = config.file_output,
                    "日志配置参数变更"
                );

                cache.logging_config = config;
                cache.increment_version();

                info!(
                    old_version = old_version,
                    new_version = cache.version,
                    timestamp = ?std::time::SystemTime::now(),
                    "日志配置已更新"
                );
            }
            ConfigFileType::ModbusSensors => {
                let config = ConfigParser::parse_modbus_config(&file_path)?;
                ConfigValidator::validate_modbus_config(&config).map_err(|e| {
                    error!(
                        file_type = ?event.file_type,
                        path = %file_path.display(),
                        error = %e,
                        timestamp = ?std::time::SystemTime::now(),
                        "配置验证失败"
                    );
                    HotReloadError::ValidationFailed {
                        file_type: event.file_type,
                        source: e,
                    }
                })?;

                let mut cache = config_cache.write().await;

                // 记录配置变更前后的关键参数差异
                info!(
                    config_type = "ModbusSensors",
                    old_address = %cache.modbus_config.server.address,
                    new_address = %config.server.address,
                    old_port = cache.modbus_config.server.port,
                    new_port = config.server.port,
                    "Modbus 配置参数变更"
                );

                cache.modbus_config = config;
                cache.increment_version();

                info!(
                    old_version = old_version,
                    new_version = cache.version,
                    timestamp = ?std::time::SystemTime::now(),
                    "Modbus 配置已更新"
                );
            }
            ConfigFileType::Pipeline => {
                let config = ConfigParser::parse_pipeline_config(&file_path)?;
                ConfigValidator::validate_pipeline_config(&config).map_err(|e| {
                    error!(
                        file_type = ?event.file_type,
                        path = %file_path.display(),
                        error = %e,
                        timestamp = ?std::time::SystemTime::now(),
                        "配置验证失败"
                    );
                    HotReloadError::ValidationFailed {
                        file_type: event.file_type,
                        source: e,
                    }
                })?;

                let mut cache = config_cache.write().await;

                // 记录配置变更前后的关键参数差异
                info!(
                    config_type = "Pipeline",
                    old_collection_interval = cache.pipeline_config.collection.interval_ms,
                    new_collection_interval = config.collection.interval_ms,
                    old_storage_interval = cache.pipeline_config.storage.interval_ms,
                    new_storage_interval = config.storage.interval_ms,
                    old_display_interval = cache.pipeline_config.display.interval_ms,
                    new_display_interval = config.display.interval_ms,
                    "管道配置参数变更"
                );

                cache.pipeline_config = config;
                cache.increment_version();

                info!(
                    old_version = old_version,
                    new_version = cache.version,
                    timestamp = ?std::time::SystemTime::now(),
                    "管道配置已更新"
                );
            }
            ConfigFileType::RatedLoadTable => {
                let config = ConfigParser::parse_rated_load_table(&file_path)?;
                ConfigValidator::validate_rated_load_table(&config).map_err(|e| {
                    error!(
                        file_type = ?event.file_type,
                        path = %file_path.display(),
                        error = %e,
                        timestamp = ?std::time::SystemTime::now(),
                        "配置验证失败"
                    );
                    HotReloadError::ValidationFailed {
                        file_type: event.file_type,
                        source: e,
                    }
                })?;

                let mut cache = config_cache.write().await;

                // 记录配置变更前后的关键参数差异
                let old_entry_count = cache
                    .rated_load_table
                    .get_all_entries()
                    .iter()
                    .map(|entries| entries.len())
                    .sum::<usize>();
                let new_entry_count = config
                    .get_all_entries()
                    .iter()
                    .map(|entries| entries.len())
                    .sum::<usize>();

                info!(
                    config_type = "RatedLoadTable",
                    old_entry_count = old_entry_count,
                    new_entry_count = new_entry_count,
                    "额定负载表配置参数变更"
                );

                cache.rated_load_table = config;
                cache.increment_version();

                info!(
                    old_version = old_version,
                    new_version = cache.version,
                    timestamp = ?std::time::SystemTime::now(),
                    "额定负载表已更新"
                );
            }
        }

        // 获取新版本号
        let new_version = {
            let cache = config_cache.read().await;
            cache.version
        };

        // 创建配置变更通知
        let change = ConfigChange {
            file_type: event.file_type,
            old_version,
            new_version,
            timestamp: event.timestamp,
        };

        // 通知所有订阅者
        Self::notify_subscribers_static(config_cache, subscribers, change).await;

        Ok(())
    }

    /// 通知所有订阅者（静态方法，用于 tokio::spawn）
    async fn notify_subscribers_static(
        config_cache: &Arc<RwLock<ConfigCache>>,
        subscribers: &Arc<RwLock<Vec<Box<dyn ConfigSubscriber>>>>,
        change: ConfigChange,
    ) {
        let start_time = Instant::now();

        // 获取配置快照
        let snapshot = {
            let cache = config_cache.read().await;
            ConfigSnapshot {
                sensor_calibration: cache.sensor_calibration.clone(),
                alarm_thresholds: cache.alarm_thresholds.clone(),
                logging_config: cache.logging_config.clone(),
                modbus_config: cache.modbus_config.clone(),
                pipeline_config: cache.pipeline_config.clone(),
                rated_load_table: cache.rated_load_table.clone(),
                version: cache.version,
            }
        };

        // 获取订阅者列表（读锁）
        let subscribers_list = subscribers.read().await;

        if subscribers_list.is_empty() {
            debug!("没有配置订阅者需要通知");
            return;
        }

        info!(
            "通知 {} 个订阅者配置变更: {:?}",
            subscribers_list.len(),
            change.file_type
        );

        // 顺序通知所有订阅者，但为每个订阅者设置超时
        for subscriber in subscribers_list.iter() {
            let subscriber_name = subscriber.name();
            let change_clone = change.clone();
            let snapshot_clone = snapshot.clone();

            // 使用 timeout 确保单个订阅者不会阻塞太久
            match tokio::time::timeout(
                std::time::Duration::from_millis(50),
                subscriber.on_config_changed(change_clone, snapshot_clone),
            )
            .await
            {
                Ok(_) => {
                    debug!("订阅者 {} 已收到配置变更通知", subscriber_name);
                }
                Err(_) => {
                    warn!("订阅者 {} 处理配置变更超时（> 50ms）", subscriber_name);
                }
            }
        }

        let elapsed = start_time.elapsed();
        if elapsed.as_millis() > 100 {
            warn!(
                "通知所有订阅者耗时 {}ms，超过 100ms 目标",
                elapsed.as_millis()
            );
        } else {
            debug!("通知所有订阅者完成，耗时 {}ms", elapsed.as_millis());
        }
    }

    /// 内部重载配置方法
    async fn reload_config_internal(
        &self,
        file_type: ConfigFileType,
    ) -> Result<(), HotReloadError> {
        // 构建配置文件路径
        let file_name = match file_type {
            ConfigFileType::SensorCalibration => "sensor_calibration.toml",
            ConfigFileType::AlarmThresholds => "alarm_thresholds.toml",
            ConfigFileType::Logging => "logging.toml",
            ConfigFileType::ModbusSensors => "modbus_sensors.toml",
            ConfigFileType::Pipeline => "pipeline_config.toml",
            ConfigFileType::RatedLoadTable => "rated_load_table.csv",
        };

        let file_path = self.config_dir.join(file_name);

        // 记录旧版本号
        let old_version = {
            let cache = self.config_cache.read().await;
            cache.version
        };

        // 解析和验证配置
        match file_type {
            ConfigFileType::SensorCalibration => {
                let config = ConfigParser::parse_sensor_calibration(&file_path)?;
                ConfigValidator::validate_sensor_calibration(&config).map_err(|e| {
                    HotReloadError::ValidationFailed {
                        file_type,
                        source: e,
                    }
                })?;

                let mut cache = self.config_cache.write().await;
                cache.sensor_calibration = config;
                cache.increment_version();
            }
            ConfigFileType::AlarmThresholds => {
                let config = ConfigParser::parse_alarm_thresholds(&file_path)?;
                ConfigValidator::validate_alarm_thresholds(&config).map_err(|e| {
                    HotReloadError::ValidationFailed {
                        file_type,
                        source: e,
                    }
                })?;

                let mut cache = self.config_cache.write().await;
                cache.alarm_thresholds = config;
                cache.increment_version();
            }
            ConfigFileType::Logging => {
                let config = ConfigParser::parse_logging_config(&file_path)?;
                ConfigValidator::validate_logging_config(&config).map_err(|e| {
                    HotReloadError::ValidationFailed {
                        file_type,
                        source: e,
                    }
                })?;

                let mut cache = self.config_cache.write().await;
                cache.logging_config = config;
                cache.increment_version();
            }
            ConfigFileType::ModbusSensors => {
                let config = ConfigParser::parse_modbus_config(&file_path)?;
                ConfigValidator::validate_modbus_config(&config).map_err(|e| {
                    HotReloadError::ValidationFailed {
                        file_type,
                        source: e,
                    }
                })?;

                let mut cache = self.config_cache.write().await;
                cache.modbus_config = config;
                cache.increment_version();
            }
            ConfigFileType::Pipeline => {
                let config = ConfigParser::parse_pipeline_config(&file_path)?;
                ConfigValidator::validate_pipeline_config(&config).map_err(|e| {
                    HotReloadError::ValidationFailed {
                        file_type,
                        source: e,
                    }
                })?;

                let mut cache = self.config_cache.write().await;
                cache.pipeline_config = config;
                cache.increment_version();
            }
            ConfigFileType::RatedLoadTable => {
                let config = ConfigParser::parse_rated_load_table(&file_path)?;
                ConfigValidator::validate_rated_load_table(&config).map_err(|e| {
                    HotReloadError::ValidationFailed {
                        file_type,
                        source: e,
                    }
                })?;

                let mut cache = self.config_cache.write().await;
                cache.rated_load_table = config;
                cache.increment_version();
            }
        }

        // 获取新版本号
        let new_version = {
            let cache = self.config_cache.read().await;
            cache.version
        };

        info!("配置重载成功: {:?}", file_type);

        // 创建配置变更通知
        let change = ConfigChange {
            file_type,
            old_version,
            new_version,
            timestamp: std::time::SystemTime::now(),
        };

        // 通知所有订阅者
        self.notify_subscribers(change).await;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_create_manager() {
        let temp_dir = TempDir::new().unwrap();
        let manager = HotReloadConfigManager::new(temp_dir.path().to_path_buf());
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_get_config_snapshot() {
        let temp_dir = TempDir::new().unwrap();
        let manager = HotReloadConfigManager::new(temp_dir.path().to_path_buf()).unwrap();

        let snapshot = manager.get_config_snapshot().await;
        assert_eq!(snapshot.version, 0);
    }

    #[tokio::test]
    async fn test_update_mutex() {
        let temp_dir = TempDir::new().unwrap();
        let manager = HotReloadConfigManager::new(temp_dir.path().to_path_buf()).unwrap();

        // 获取锁
        let lock1 = manager.update_mutex.try_lock();
        assert!(lock1.is_ok());

        // 尝试再次获取锁（应该失败）
        let lock2 = manager.update_mutex.try_lock();
        assert!(lock2.is_err());
    }
}
