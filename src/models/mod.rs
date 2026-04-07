// 数据模型模块

pub mod sensor_data;
pub mod processed_data;
pub mod sensor_calibration;
pub mod rated_load_table;
pub mod crane_config;
pub mod alarm_record;

// 重新导出常用类型
pub use sensor_data::SensorData;
pub use processed_data::ProcessedData;
pub use sensor_calibration::{SensorCalibration, SensorCalibrationParams, AlarmThresholds, AngleThresholds, MomentThresholds};
pub use rated_load_table::{RatedLoadTable, RatedLoadEntry};
pub use crane_config::CraneConfig;
pub use alarm_record::{AlarmRecord, AlarmType};
