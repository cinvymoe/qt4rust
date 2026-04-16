// src/pipeline/config_provider.rs

use crate::models::rated_load_table::RatedLoadTable;
use sensor_core::{AlarmThresholds, SensorCalibration};
use std::sync::RwLock;
use tracing::{info, warn};

/// Unified configuration provider for all pipelines.
///
/// Centrally manages shared configuration:
/// - SensorCalibration: Sensor calibration parameters
/// - RatedLoadTable: Rated load table
/// - AlarmThresholds: Alarm thresholds
///
/// All pipelines get configuration via Arc<ConfigProvider>,
/// configuration changes only need to update one place,
/// all consumers automatically take effect.
pub struct ConfigProvider {
    sensor_calibration: RwLock<SensorCalibration>,
    rated_load_table: RwLock<RatedLoadTable>,
    alarm_thresholds: RwLock<AlarmThresholds>,
}

impl ConfigProvider {
    pub fn new() -> Self {
        Self {
            sensor_calibration: RwLock::new(SensorCalibration::default()),
            rated_load_table: RwLock::new(RatedLoadTable::default()),
            alarm_thresholds: RwLock::new(AlarmThresholds::default()),
        }
    }

    pub fn with_config(
        sensor_calibration: SensorCalibration,
        rated_load_table: RatedLoadTable,
        alarm_thresholds: AlarmThresholds,
    ) -> Self {
        Self {
            sensor_calibration: RwLock::new(sensor_calibration),
            rated_load_table: RwLock::new(rated_load_table),
            alarm_thresholds: RwLock::new(alarm_thresholds),
        }
    }

    pub fn get_sensor_calibration(&self) -> Result<SensorCalibration, String> {
        self.sensor_calibration
            .read()
            .map(|guard| guard.clone())
            .map_err(|e| format!("Failed to read sensor_calibration: {}", e))
    }

    pub fn get_sensor_calibration_arc(&self) -> std::sync::Arc<RwLock<SensorCalibration>> {
        std::sync::Arc::new(RwLock::new(self.sensor_calibration.read().unwrap().clone()))
    }

    pub fn update_sensor_calibration(&self, calibration: SensorCalibration) {
        match self.sensor_calibration.write() {
            Ok(mut guard) => {
                *guard = calibration;
                info!("SensorCalibration updated");
            }
            Err(e) => {
                warn!("Failed to update sensor_calibration: {}", e);
            }
        }
    }

    pub fn get_rated_load_table(&self) -> Result<RatedLoadTable, String> {
        self.rated_load_table
            .read()
            .map(|guard| guard.clone())
            .map_err(|e| format!("Failed to read rated_load_table: {}", e))
    }

    pub fn get_rated_load_table_arc(&self) -> std::sync::Arc<RwLock<RatedLoadTable>> {
        std::sync::Arc::new(RwLock::new(self.rated_load_table.read().unwrap().clone()))
    }

    pub fn update_rated_load_table(&self, table: RatedLoadTable) {
        match self.rated_load_table.write() {
            Ok(mut guard) => {
                *guard = table;
                info!("RatedLoadTable updated");
            }
            Err(e) => {
                warn!("Failed to update rated_load_table: {}", e);
            }
        }
    }

    pub fn get_alarm_thresholds(&self) -> Result<AlarmThresholds, String> {
        self.alarm_thresholds
            .read()
            .map(|guard| guard.clone())
            .map_err(|e| format!("Failed to read alarm_thresholds: {}", e))
    }

    pub fn get_alarm_thresholds_arc(&self) -> std::sync::Arc<RwLock<AlarmThresholds>> {
        std::sync::Arc::new(RwLock::new(self.alarm_thresholds.read().unwrap().clone()))
    }

    pub fn update_alarm_thresholds(&self, thresholds: AlarmThresholds) {
        match self.alarm_thresholds.write() {
            Ok(mut guard) => {
                *guard = thresholds;
                info!("AlarmThresholds updated");
            }
            Err(e) => {
                warn!("Failed to update alarm_thresholds: {}", e);
            }
        }
    }

    pub fn update_all(
        &self,
        sensor_calibration: SensorCalibration,
        rated_load_table: RatedLoadTable,
        alarm_thresholds: AlarmThresholds,
    ) {
        self.update_sensor_calibration(sensor_calibration);
        self.update_rated_load_table(rated_load_table);
        self.update_alarm_thresholds(alarm_thresholds);
        info!("All configs updated");
    }
}

impl Default for ConfigProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_config_provider_creation() {
        let provider = ConfigProvider::new();
        assert!(provider.get_sensor_calibration().is_ok());
    }

    #[test]
    fn test_config_provider_update() {
        let provider = ConfigProvider::new();

        let mut calibration = provider.get_sensor_calibration().unwrap();
        calibration.weight.zero_ad = 100.0;

        // 更新后应该能获取新值
        provider.update_sensor_calibration(calibration.clone());
        let updated = provider.get_sensor_calibration().unwrap();
        assert!((updated.weight.zero_ad - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_config_provider_thread_safety() {
        let provider = Arc::new(ConfigProvider::new());
        let provider_clone = Arc::clone(&provider);

        let handle = std::thread::spawn(move || {
            let cal = provider_clone.get_sensor_calibration().unwrap();
            provider_clone.update_sensor_calibration(cal);
        });

        handle.join().unwrap();
        assert!(provider.get_sensor_calibration().is_ok());
    }
}
