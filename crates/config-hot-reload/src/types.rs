//! 公共类型定义

use std::path::PathBuf;
use std::time::SystemTime;

use qt_rust_demo::config::pipeline_config::PipelineConfig;
use qt_rust_demo::logging::config::LogConfig;
use qt_rust_demo::models::rated_load_table::RatedLoadTable;
use sensor_core::{AlarmThresholds, SensorCalibration};

use crate::parser::ModbusConfig;

/// 配置文件类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigFileType {
    SensorCalibration,
    AlarmThresholds,
    Logging,
    ModbusSensors,
    Pipeline,
    RatedLoadTable,
}

/// 配置文件变更事件
#[derive(Debug, Clone)]
pub struct ConfigFileEvent {
    pub file_type: ConfigFileType,
    pub path: PathBuf,
    pub timestamp: SystemTime,
}

/// 配置变更通知
#[derive(Debug, Clone)]
pub struct ConfigChange {
    pub file_type: ConfigFileType,
    pub old_version: u64,
    pub new_version: u64,
    pub timestamp: SystemTime,
}

/// 配置快照（只读）
///
/// 包含所有配置类型的完整快照，用于订阅者获取当前配置状态。
#[derive(Debug, Clone)]
pub struct ConfigSnapshot {
    pub sensor_calibration: SensorCalibration,
    pub alarm_thresholds: AlarmThresholds,
    pub logging_config: LogConfig,
    pub modbus_config: ModbusConfig,
    pub pipeline_config: PipelineConfig,
    pub rated_load_table: RatedLoadTable,
    pub version: u64,
}

impl Default for ConfigSnapshot {
    fn default() -> Self {
        Self {
            sensor_calibration: SensorCalibration::default(),
            alarm_thresholds: AlarmThresholds::default(),
            logging_config: LogConfig::default(),
            modbus_config: ModbusConfig::default(),
            pipeline_config: PipelineConfig::default(),
            rated_load_table: RatedLoadTable::default(),
            version: 0,
        }
    }
}
