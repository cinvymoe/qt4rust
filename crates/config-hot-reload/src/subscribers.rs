use async_trait::async_trait;
use std::sync::Arc;
use std::sync::RwLock;
use tracing::{error, info, warn};

use crate::parser::ModbusConfig;
use crate::subscriber::ConfigSubscriber;
use crate::types::{ConfigChange, ConfigFileType, ConfigSnapshot};
use qt_rust_demo::config::pipeline_config::PipelineConfig;
use qt_rust_demo::logging::config::LogConfig;
use qt_rust_demo::models::rated_load_table::RatedLoadTable;
use sensor_core::{AlarmThresholds, SensorCalibration};

/// Pipeline 配置订阅者 - 在下一周期应用新的采集/存储/显示配置
pub struct PipelineConfigSubscriber {
    config: Arc<RwLock<PipelineConfig>>,
}

impl PipelineConfigSubscriber {
    pub fn new(config: Arc<RwLock<PipelineConfig>>) -> Self {
        Self { config }
    }
}

#[async_trait]
impl ConfigSubscriber for PipelineConfigSubscriber {
    async fn on_config_changed(&self, change: ConfigChange, snapshot: ConfigSnapshot) {
        if change.file_type != ConfigFileType::Pipeline {
            return;
        }

        info!(
            subscriber = "PipelineConfigSubscriber",
            old_version = change.old_version,
            new_version = change.new_version,
            "收到管道配置变更通知"
        );

        match self.config.write() {
            Ok(mut guard) => {
                let old_interval = guard.collection.interval_ms;
                let old_storage_interval = guard.storage.interval_ms;
                let old_display_interval = guard.display.interval_ms;
                let old_buffer_size = guard.collection.buffer_size;
                let old_batch_size = guard.storage.batch_size;
                let old_filter =
                    format!("{}/{}", guard.filter.filter_type, guard.filter.window_size);

                *guard = snapshot.pipeline_config.clone();
                let new_config = &*guard;
                let new_filter = format!(
                    "{}/{}",
                    new_config.filter.filter_type, new_config.filter.window_size
                );

                info!(
                    subscriber = "PipelineConfigSubscriber",
                    collection_interval = format!(
                        "{}ms -> {}ms",
                        old_interval, new_config.collection.interval_ms
                    ),
                    storage_interval = format!(
                        "{}ms -> {}ms",
                        old_storage_interval, new_config.storage.interval_ms
                    ),
                    display_interval = format!(
                        "{}ms -> {}ms",
                        old_display_interval, new_config.display.interval_ms
                    ),
                    buffer_size = format!(
                        "{} -> {}",
                        old_buffer_size, new_config.collection.buffer_size
                    ),
                    batch_size = format!("{} -> {}", old_batch_size, new_config.storage.batch_size),
                    filter = format!("{} -> {}", old_filter, new_filter),
                    "管道配置已更新，将在下一周期生效"
                );
            }
            Err(e) => {
                error!(subscriber = "PipelineConfigSubscriber", error = %e, "获取管道配置写锁失败");
            }
        }
    }

    fn name(&self) -> &str {
        "PipelineConfigSubscriber"
    }
}

/// DataProcessingSubscriber - handles SensorCalibration and RatedLoadTable config changes
pub struct DataProcessingSubscriber {
    sensor_calibration: Arc<RwLock<SensorCalibration>>,
    rated_load_table: Arc<RwLock<RatedLoadTable>>,
}

impl DataProcessingSubscriber {
    pub fn new(
        sensor_calibration: Arc<RwLock<SensorCalibration>>,
        rated_load_table: Arc<RwLock<RatedLoadTable>>,
    ) -> Self {
        Self {
            sensor_calibration,
            rated_load_table,
        }
    }
}

#[async_trait]
impl ConfigSubscriber for DataProcessingSubscriber {
    async fn on_config_changed(&self, change: ConfigChange, snapshot: ConfigSnapshot) {
        match change.file_type {
            ConfigFileType::SensorCalibration => {
                info!(
                    subscriber = "DataProcessingSubscriber",
                    old_version = change.old_version,
                    new_version = change.new_version,
                    "收到传感器校准配置变更通知"
                );

                match self.sensor_calibration.write() {
                    Ok(mut guard) => {
                        let old_weight = guard.weight().scale_value;
                        let old_weight_multiplier = guard.weight().multiplier;
                        let old_angle = guard.angle().scale_value;
                        let old_radius = guard.radius().scale_value;

                        *guard = snapshot.sensor_calibration.clone();
                        let new_cal = &*guard;

                        info!(
                            subscriber = "DataProcessingSubscriber",
                            weight_scale =
                                format!("{} -> {}", old_weight, new_cal.weight().scale_value),
                            weight_multiplier = format!(
                                "{} -> {}",
                                old_weight_multiplier, new_cal.weight().multiplier
                            ),
                            angle_scale = format!("{} -> {}", old_angle, new_cal.angle().scale_value),
                            radius_scale =
                                format!("{} -> {}", old_radius, new_cal.radius().scale_value),
                            "🔄 传感器校准配置已更新，将在下一次数据转换时生效"
                        );

                        info!("📝 [新标定参数] weight: zero_ad={:.2}, zero_value={:.2}, scale_ad={:.2}, scale_value={:.2}, multiplier={:.2}",
                            new_cal.weight().zero_ad,
                            new_cal.weight().zero_value,
                            new_cal.weight().scale_ad,
                            new_cal.weight().scale_value,
                            new_cal.weight().multiplier);
                    }
                    Err(e) => {
                        error!(subscriber = "DataProcessingSubscriber", error = %e, "获取传感器校准配置写锁失败");
                    }
                }
            }
            ConfigFileType::RatedLoadTable => {
                info!(
                    subscriber = "DataProcessingSubscriber",
                    old_version = change.old_version,
                    new_version = change.new_version,
                    "收到额定负载表变更通知"
                );

                match self.rated_load_table.write() {
                    Ok(mut guard) => {
                        let old_count = guard.len();
                        let old_warning = guard.moment_warning_threshold;
                        let old_alarm = guard.moment_alarm_threshold;

                        *guard = snapshot.rated_load_table.clone();
                        let new_table = &*guard;

                        info!(
                            subscriber = "DataProcessingSubscriber",
                            entries = format!("{} -> {}", old_count, new_table.len()),
                            warning = format!(
                                "{}% -> {}%",
                                old_warning, new_table.moment_warning_threshold
                            ),
                            alarm =
                                format!("{}% -> {}%", old_alarm, new_table.moment_alarm_threshold),
                            "额定负载表已更新，将在下一次力矩计算时生效"
                        );
                    }
                    Err(e) => {
                        error!(subscriber = "DataProcessingSubscriber", error = %e, "获取额定负载表写锁失败");
                    }
                }
            }
            _ => {}
        }
    }

    fn name(&self) -> &str {
        "DataProcessingSubscriber"
    }
}

/// AlarmDetectionSubscriber - applies new alarm thresholds immediately
pub struct AlarmDetectionSubscriber {
    alarm_thresholds: Arc<RwLock<AlarmThresholds>>,
}

impl AlarmDetectionSubscriber {
    pub fn new(alarm_thresholds: Arc<RwLock<AlarmThresholds>>) -> Self {
        Self { alarm_thresholds }
    }
}

#[async_trait]
impl ConfigSubscriber for AlarmDetectionSubscriber {
    async fn on_config_changed(&self, change: ConfigChange, snapshot: ConfigSnapshot) {
        if change.file_type != ConfigFileType::AlarmThresholds {
            return;
        }

        info!(
            subscriber = "AlarmDetectionSubscriber",
            old_version = change.old_version,
            new_version = change.new_version,
            "收到报警阈值配置变更通知"
        );

        match self.alarm_thresholds.write() {
            Ok(mut guard) => {
                let old_warning = guard.moment.warning_percentage;
                let old_alarm = guard.moment.alarm_percentage;

                *guard = snapshot.alarm_thresholds.clone();
                let new_thresh = &*guard;

                let threshold_lowered = new_thresh.moment.warning_percentage < old_warning
                    || new_thresh.moment.alarm_percentage < old_alarm;

                info!(
                    subscriber = "AlarmDetectionSubscriber",
                    warning = format!(
                        "{}% -> {}%",
                        old_warning, new_thresh.moment.warning_percentage
                    ),
                    alarm = format!("{}% -> {}%", old_alarm, new_thresh.moment.alarm_percentage),
                    "报警阈值已更新，立即生效"
                );

                if threshold_lowered {
                    warn!(
                        subscriber = "AlarmDetectionSubscriber",
                        "阈值降低，可能导致新的报警触发"
                    );
                }
            }
            Err(e) => {
                error!(subscriber = "AlarmDetectionSubscriber", error = %e, "获取报警阈值写锁失败");
            }
        }
    }

    fn name(&self) -> &str {
        "AlarmDetectionSubscriber"
    }
}

/// LoggingConfigSubscriber - applies new log levels and output config immediately
pub struct LoggingConfigSubscriber {
    log_config: Arc<RwLock<LogConfig>>,
}

impl LoggingConfigSubscriber {
    pub fn new(log_config: Arc<RwLock<LogConfig>>) -> Self {
        Self { log_config }
    }
}

#[async_trait]
impl ConfigSubscriber for LoggingConfigSubscriber {
    async fn on_config_changed(&self, change: ConfigChange, snapshot: ConfigSnapshot) {
        if change.file_type != ConfigFileType::Logging {
            return;
        }

        info!(
            subscriber = "LoggingConfigSubscriber",
            old_version = change.old_version,
            new_version = change.new_version,
            "收到日志配置变更通知"
        );

        match self.log_config.write() {
            Ok(mut guard) => {
                let new_config = snapshot.logging_config.clone();
                let old_level = format!("{:?}", guard.default_level);
                let old_console = guard.console_output;
                let old_file = guard.file_output;

                *guard = new_config;
                let updated = &*guard;
                let new_level = format!("{:?}", updated.default_level);

                info!(
                    subscriber = "LoggingConfigSubscriber",
                    level = format!("{} -> {}", old_level, new_level),
                    console = format!("{} -> {}", old_console, updated.console_output),
                    file = format!("{} -> {}", old_file, updated.file_output),
                    "日志配置已更新，立即生效"
                );
            }
            Err(e) => {
                error!(subscriber = "LoggingConfigSubscriber", error = %e, "获取日志配置写锁失败");
            }
        }
    }

    fn name(&self) -> &str {
        "LoggingConfigSubscriber"
    }
}

/// SensorDataSourceSubscriber - handles Modbus config changes, sets reconnect flag
pub struct SensorDataSourceSubscriber {
    modbus_config: Arc<RwLock<ModbusConfig>>,
    reconnect_flag: Arc<RwLock<bool>>,
}

impl SensorDataSourceSubscriber {
    pub fn new(
        modbus_config: Arc<RwLock<ModbusConfig>>,
        reconnect_flag: Arc<RwLock<bool>>,
    ) -> Self {
        Self {
            modbus_config,
            reconnect_flag,
        }
    }
}

#[async_trait]
impl ConfigSubscriber for SensorDataSourceSubscriber {
    async fn on_config_changed(&self, change: ConfigChange, snapshot: ConfigSnapshot) {
        if change.file_type != ConfigFileType::ModbusSensors {
            return;
        }

        info!(
            subscriber = "SensorDataSourceSubscriber",
            old_version = change.old_version,
            new_version = change.new_version,
            "收到 Modbus 配置变更通知"
        );

        match self.modbus_config.write() {
            Ok(mut guard) => {
                let old_address = guard.server.address.clone();
                let old_port = guard.server.port;

                *guard = snapshot.modbus_config.clone();
                let new_cfg = &*guard;

                info!(
                    subscriber = "SensorDataSourceSubscriber",
                    address = format!("{} -> {}", old_address, new_cfg.server.address),
                    port = format!("{} -> {}", old_port, new_cfg.server.port),
                    sensor_count = new_cfg.sensors.len(),
                    "Modbus 配置已更新"
                );

                match self.reconnect_flag.write() {
                    Ok(mut flag) => {
                        *flag = true;
                        info!(
                            subscriber = "SensorDataSourceSubscriber",
                            "已设置重连标志，传感器数据源将在下次采集时重连"
                        );
                    }
                    Err(e) => {
                        error!(subscriber = "SensorDataSourceSubscriber", error = %e, "设置重连标志失败");
                    }
                }
            }
            Err(e) => {
                error!(subscriber = "SensorDataSourceSubscriber", error = %e, "获取 Modbus 配置写锁失败");
            }
        }
    }

    fn name(&self) -> &str {
        "SensorDataSourceSubscriber"
    }
}

/// Shared config references wrapped in Arc<RwLock<>> for thread-safe config updates
#[derive(Clone)]
pub struct SharedConfigRefs {
    pub pipeline_config: Arc<RwLock<PipelineConfig>>,
    pub sensor_calibration: Arc<RwLock<SensorCalibration>>,
    pub rated_load_table: Arc<RwLock<RatedLoadTable>>,
    pub alarm_thresholds: Arc<RwLock<AlarmThresholds>>,
    pub log_config: Arc<RwLock<LogConfig>>,
    pub modbus_config: Arc<RwLock<ModbusConfig>>,
    pub modbus_reconnect_flag: Arc<RwLock<bool>>,
}

impl Default for SharedConfigRefs {
    fn default() -> Self {
        Self {
            pipeline_config: Arc::new(RwLock::new(PipelineConfig::default())),
            sensor_calibration: Arc::new(RwLock::new(SensorCalibration::default())),
            rated_load_table: Arc::new(RwLock::new(RatedLoadTable::default())),
            alarm_thresholds: Arc::new(RwLock::new(AlarmThresholds::default())),
            log_config: Arc::new(RwLock::new(LogConfig::default())),
            modbus_config: Arc::new(RwLock::new(ModbusConfig::default())),
            modbus_reconnect_flag: Arc::new(RwLock::new(false)),
        }
    }
}

impl SharedConfigRefs {
    pub fn new(
        pipeline_config: PipelineConfig,
        sensor_calibration: SensorCalibration,
        rated_load_table: RatedLoadTable,
        alarm_thresholds: AlarmThresholds,
        log_config: LogConfig,
        modbus_config: ModbusConfig,
    ) -> Self {
        Self {
            pipeline_config: Arc::new(RwLock::new(pipeline_config)),
            sensor_calibration: Arc::new(RwLock::new(sensor_calibration)),
            rated_load_table: Arc::new(RwLock::new(rated_load_table)),
            alarm_thresholds: Arc::new(RwLock::new(alarm_thresholds)),
            log_config: Arc::new(RwLock::new(log_config)),
            modbus_config: Arc::new(RwLock::new(modbus_config)),
            modbus_reconnect_flag: Arc::new(RwLock::new(false)),
        }
    }
}

pub async fn register_all_subscribers(
    manager: &mut crate::HotReloadConfigManager,
    shared_refs: &SharedConfigRefs,
) {
    manager
        .subscribe(Box::new(PipelineConfigSubscriber::new(Arc::clone(
            &shared_refs.pipeline_config,
        ))))
        .await;
    manager
        .subscribe(Box::new(DataProcessingSubscriber::new(
            Arc::clone(&shared_refs.sensor_calibration),
            Arc::clone(&shared_refs.rated_load_table),
        )))
        .await;
    manager
        .subscribe(Box::new(AlarmDetectionSubscriber::new(Arc::clone(
            &shared_refs.alarm_thresholds,
        ))))
        .await;
    manager
        .subscribe(Box::new(LoggingConfigSubscriber::new(Arc::clone(
            &shared_refs.log_config,
        ))))
        .await;
    manager
        .subscribe(Box::new(SensorDataSourceSubscriber::new(
            Arc::clone(&shared_refs.modbus_config),
            Arc::clone(&shared_refs.modbus_reconnect_flag),
        )))
        .await;

    info!("所有配置订阅者已注册");
}
