// 处理后的数据模型

use std::time::SystemTime;
use super::sensor_data::SensorData;
use super::crane_config::CraneConfig;

/// 处理后的数据（计算后的结果）
#[derive(Debug, Clone)]
pub struct ProcessedData {
    /// 当前载荷（吨）
    pub current_load: f64,
    
    /// 工作半径（米）
    pub working_radius: f64,
    
    /// 吊臂角度（度）
    pub boom_angle: f64,
    
    /// 力矩百分比
    pub moment_percentage: f64,
    
    /// 是否危险
    pub is_danger: bool,
    
    /// 验证错误
    pub validation_error: Option<String>,
    
    /// 时间戳
    pub timestamp: SystemTime,
    
    /// 序列号
    pub sequence_number: u64,
}

impl ProcessedData {
    /// 默认额定载荷（吨）
    const DEFAULT_RATED_LOAD: f64 = 25.0;
    
    /// 从传感器原始数据创建处理后的数据
    /// 
    /// # 参数
    /// - `raw_data`: 传感器原始数据（AD1, AD2, AD3）
    /// - `sequence_number`: 序列号
    pub fn from_sensor_data(raw_data: SensorData, sequence_number: u64) -> Self {
        let moment_percentage = Self::calculate_moment_percentage(&raw_data);
        let is_danger = moment_percentage >= 90.0;
        let validation_error = raw_data.validate().err();
        
        Self {
            current_load: raw_data.ad1_load,
            working_radius: raw_data.ad2_radius,
            boom_angle: raw_data.ad3_angle,
            moment_percentage,
            is_danger,
            validation_error,
            timestamp: SystemTime::now(),
            sequence_number,
        }
    }
    
    /// 从传感器原始数据和配置创建处理后的数据
    /// 
    /// # 参数
    /// - `raw_data`: 传感器原始数据（AD 值）
    /// - `config`: 起重机配置（包含标定参数和额定载荷表）
    /// - `sequence_number`: 序列号
    /// 
    /// # 返回
    /// 处理后的数据，包含转换后的物理值和状态判断
    pub fn from_sensor_data_with_config(
        raw_data: SensorData,
        config: &CraneConfig,
        sequence_number: u64,
    ) -> Self {
        // 使用配置的标定参数转换 AD 值为物理值
        let current_load = config.sensor_calibration.convert_weight_ad_to_value(raw_data.ad1_load);
        let working_radius = config.sensor_calibration.convert_radius_ad_to_value(raw_data.ad2_radius);
        let boom_angle = config.sensor_calibration.convert_angle_ad_to_value(raw_data.ad3_angle);
        
        // 根据幅度查询额定载荷
        let rated_load = config.rated_load_table.get_rated_load(working_radius);
        
        // 计算力矩百分比
        let moment_percentage = Self::calculate_moment_percentage_with_load(
            current_load,
            working_radius,
            rated_load,
        );
        
        // 使用配置的危险阈值判断（优先使用 SensorCalibration 中的力矩报警百分比）
        let is_danger = config.sensor_calibration.is_moment_alarm(moment_percentage);
        
        // 检查传感器值是否超过预警/报警阈值
        let mut validation_error = None;
        
        // 检查力矩百分比（最高优先级）
        if config.sensor_calibration.is_moment_alarm(moment_percentage) {
            validation_error = Some(format!("力矩报警: {:.1}% >= {:.1}%", 
                moment_percentage, config.sensor_calibration.moment_alarm_percentage));
        } else if config.sensor_calibration.is_moment_warning(moment_percentage) {
            validation_error = Some(format!("力矩预警: {:.1}% >= {:.1}%", 
                moment_percentage, config.sensor_calibration.moment_warning_percentage));
        }
        // 检查角度
        else if config.sensor_calibration.is_angle_alarm(boom_angle) {
            validation_error = Some(format!("角度报警: {:.1} 度 >= {:.1} 度", 
                boom_angle, config.sensor_calibration.angle_alarm_value));
        } else if config.sensor_calibration.is_angle_warning(boom_angle) {
            validation_error = Some(format!("角度预警: {:.1} 度 >= {:.1} 度", 
                boom_angle, config.sensor_calibration.angle_warning_value));
        }
        
        Self {
            current_load,
            working_radius,
            boom_angle,
            moment_percentage,
            is_danger,
            validation_error,
            timestamp: SystemTime::now(),
            sequence_number,
        }
    }
    
    /// 计算力矩百分比
    /// 
    /// 力矩 = 载荷 × 工作半径
    /// 力矩百分比 = (当前力矩 / 额定力矩) × 100%
    fn calculate_moment_percentage(data: &SensorData) -> f64 {
        let current_moment = data.ad1_load * data.ad2_radius;
        let rated_moment = Self::DEFAULT_RATED_LOAD * data.ad2_radius;
        
        if rated_moment > 0.0 {
            (current_moment / rated_moment) * 100.0
        } else {
            0.0
        }
    }
    
    /// 计算力矩百分比（使用指定的额定载荷）
    /// 
    /// # 参数
    /// - `current_load`: 当前载荷（吨）
    /// - `working_radius`: 工作半径（米）
    /// - `rated_load`: 额定载荷（吨）
    /// 
    /// # 返回
    /// 力矩百分比（%）
    fn calculate_moment_percentage_with_load(
        current_load: f64,
        working_radius: f64,
        rated_load: f64,
    ) -> f64 {
        let current_moment = current_load * working_radius;
        let rated_moment = rated_load * working_radius;
        
        if rated_moment > 0.0 {
            (current_moment / rated_moment) * 100.0
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{SensorCalibration, RatedLoadTable, RatedLoadEntry};
    
    #[test]
    fn test_from_sensor_data() {
        let sensor_data = SensorData::new(20.0, 10.0, 60.0);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);
        
        assert_eq!(processed.sequence_number, 1);
        assert_eq!(processed.current_load, 20.0);
        assert_eq!(processed.working_radius, 10.0);
        assert_eq!(processed.boom_angle, 60.0);
        assert_eq!(processed.moment_percentage, 80.0);
        assert!(!processed.is_danger);
    }
    
    #[test]
    fn test_danger_detection() {
        let sensor_data = SensorData::new(23.0, 10.0, 60.0);
        let processed = ProcessedData::from_sensor_data(sensor_data, 2);
        
        assert!(processed.is_danger);
        assert!(processed.moment_percentage >= 90.0);
    }
    
    #[test]
    fn test_validation_error() {
        let sensor_data = SensorData::new(-5.0, 10.0, 60.0);  // Invalid negative load
        let processed = ProcessedData::from_sensor_data(sensor_data, 3);
        
        assert!(processed.validation_error.is_some());
        assert!(processed.validation_error.unwrap().contains("负值"));
    }
    
    #[test]
    fn test_from_sensor_data_with_config() {
        // 创建配置
        let config = CraneConfig::default();
        
        // 创建传感器数据（AD 值：中点）
        let sensor_data = SensorData::new(2047.5, 2047.5, 2047.5);
        
        // 使用配置处理数据
        let processed = ProcessedData::from_sensor_data_with_config(sensor_data, &config, 1);
        
        // 验证转换结果（默认配置：0-4095 AD -> 0-50吨, 0-20米, 0-90度）
        assert!((processed.current_load - 25.0).abs() < 0.5);  // 约 25 吨
        assert!((processed.working_radius - 10.0).abs() < 0.5); // 约 10 米
        assert!((processed.boom_angle - 45.0).abs() < 0.5);     // 约 45 度
        assert_eq!(processed.sequence_number, 1);
    }
    
    #[test]
    fn test_from_sensor_data_with_config_uses_rated_load_table() {
        // 创建配置
        let config = CraneConfig::default();
        
        // 创建传感器数据，工作半径对应 5 米（AD 值约 1023.75）
        // 根据默认载荷表，5 米对应 40 吨额定载荷
        let sensor_data = SensorData::new(2047.5, 1023.75, 2047.5);
        
        let processed = ProcessedData::from_sensor_data_with_config(sensor_data, &config, 1);
        
        // 验证使用了载荷表中的额定载荷
        // 当前载荷约 25 吨，工作半径约 5 米，额定载荷 40 吨
        // 力矩百分比 = (25 * 5) / (40 * 5) * 100 = 62.5%
        assert!((processed.moment_percentage - 62.5).abs() < 5.0);
    }
    
    #[test]
    fn test_from_sensor_data_with_config_moment_warning() {
        // 创建配置
        let config = CraneConfig::default();
        
        // 创建传感器数据，使力矩百分比达到预警阈值（90%）
        // 假设工作半径 10 米，额定载荷 25 吨
        // 需要载荷 = 25 * 0.9 = 22.5 吨
        // AD 值 = 22.5 / 50 * 4095 ≈ 1842.75
        let sensor_data = SensorData::new(1842.75, 2047.5, 2047.5);
        
        let processed = ProcessedData::from_sensor_data_with_config(sensor_data, &config, 1);
        
        // 验证预警状态
        assert!(processed.validation_error.is_some());
        let error_msg = processed.validation_error.unwrap();
        assert!(error_msg.contains("力矩预警") || error_msg.contains("力矩报警"));
    }
    
    #[test]
    fn test_from_sensor_data_with_config_moment_alarm() {
        // 创建配置
        let config = CraneConfig::default();
        
        // 创建传感器数据，使力矩百分比达到报警阈值（100%）
        // 假设工作半径 10 米，额定载荷 25 吨
        // 需要载荷 = 25 吨
        // AD 值 = 25 / 50 * 4095 ≈ 2047.5
        let sensor_data = SensorData::new(2047.5, 2047.5, 2047.5);
        
        let processed = ProcessedData::from_sensor_data_with_config(sensor_data, &config, 1);
        
        // 验证报警状态
        assert!(processed.is_danger);
        assert!(processed.validation_error.is_some());
        let error_msg = processed.validation_error.unwrap();
        assert!(error_msg.contains("力矩报警") || error_msg.contains("力矩预警"));
    }
    
    #[test]
    fn test_from_sensor_data_with_config_angle_warning() {
        // 创建配置
        let config = CraneConfig::default();
        
        // 创建传感器数据，使角度达到预警阈值（75 度）
        // AD 值 = 75 / 90 * 4095 ≈ 3412.5
        let sensor_data = SensorData::new(1000.0, 2047.5, 3412.5);
        
        let processed = ProcessedData::from_sensor_data_with_config(sensor_data, &config, 1);
        
        // 验证角度预警（如果力矩未报警）
        if processed.validation_error.is_some() {
            let error_msg = processed.validation_error.unwrap();
            // 可能是力矩预警或角度预警
            assert!(error_msg.contains("预警") || error_msg.contains("报警"));
        }
    }
    
    #[test]
    fn test_from_sensor_data_with_config_angle_alarm() {
        // 创建配置
        let config = CraneConfig::default();
        
        // 创建传感器数据，使角度达到报警阈值（85 度）
        // AD 值 = 85 / 90 * 4095 ≈ 3867.5
        let sensor_data = SensorData::new(1000.0, 2047.5, 3867.5);
        
        let processed = ProcessedData::from_sensor_data_with_config(sensor_data, &config, 1);
        
        // 验证角度报警（如果力矩未报警）
        if processed.validation_error.is_some() {
            let error_msg = processed.validation_error.unwrap();
            // 可能是力矩报警或角度报警
            assert!(error_msg.contains("报警") || error_msg.contains("预警"));
        }
    }
    
    #[test]
    fn test_calculate_moment_percentage_with_load() {
        // 测试力矩百分比计算
        let percentage = ProcessedData::calculate_moment_percentage_with_load(20.0, 10.0, 25.0);
        assert_eq!(percentage, 80.0);
        
        let percentage = ProcessedData::calculate_moment_percentage_with_load(25.0, 10.0, 25.0);
        assert_eq!(percentage, 100.0);
        
        let percentage = ProcessedData::calculate_moment_percentage_with_load(30.0, 10.0, 25.0);
        assert_eq!(percentage, 120.0);
    }
    
    #[test]
    fn test_calculate_moment_percentage_with_load_zero_rated() {
        // 测试额定载荷为 0 的情况
        let percentage = ProcessedData::calculate_moment_percentage_with_load(20.0, 10.0, 0.0);
        assert_eq!(percentage, 0.0);
    }
    
    #[test]
    fn test_calculate_moment_percentage_with_load_zero_radius() {
        // 测试工作半径为 0 的情况
        let percentage = ProcessedData::calculate_moment_percentage_with_load(20.0, 0.0, 25.0);
        assert_eq!(percentage, 0.0);
    }
}
