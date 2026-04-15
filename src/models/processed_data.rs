// 处理后的数据模型

use super::crane_config::CraneConfig;
use crate::alarm::alarm_type::AlarmSource;
use sensor_core::SensorData;
use std::time::SystemTime;

/// 处理后的数据（计算后的结果）
#[derive(Debug, Clone)]
pub struct ProcessedData {
    /// 当前载荷（吨）
    pub current_load: f64,

    /// 额定载荷（吨，从载荷表查询得到）
    pub rated_load: f64,

    /// 工作半径（米）
    pub working_radius: f64,

    /// 吊臂角度（度）
    pub boom_angle: f64,

    /// 臂长（米）
    pub boom_length: f64,

    /// 力矩百分比
    pub moment_percentage: f64,

    /// 是否预警（达到预警阈值，>=90%）
    pub is_warning: bool,

    /// 是否危险（达到报警阈值，>=100%）
    pub is_danger: bool,

    /// 验证错误
    pub validation_error: Option<String>,

    /// 时间戳
    pub timestamp: SystemTime,

    /// 序列号
    pub sequence_number: u64,

    /// 当前活动的报警来源列表
    pub alarm_sources: Vec<AlarmSource>,

    /// 报警消息列表
    pub alarm_messages: Vec<String>,
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
        let is_warning = moment_percentage >= 90.0;
        let is_danger = moment_percentage >= 100.0;
        let validation_error = raw_data.validate().err();

        Self {
            current_load: raw_data.ad1_load,
            rated_load: Self::DEFAULT_RATED_LOAD,
            working_radius: raw_data.ad2_radius,
            boom_angle: raw_data.ad3_angle,
            boom_length: raw_data.ad2_radius, // 简化模式：臂长等于半径
            moment_percentage,
            is_warning,
            is_danger,
            validation_error,
            timestamp: SystemTime::now(),
            sequence_number,
            alarm_sources: Vec::new(),
            alarm_messages: Vec::new(),
        }
    }

    /// 从传感器原始数据和配置创建处理后的数据
    ///
    /// 新逻辑：
    /// 1. ad1 -> current_load (载荷)
    /// 2. ad2 -> boom_length (臂长)
    /// 3. ad3 -> boom_angle (臂角)
    /// 4. working_radius = boom_length * cos(boom_angle) (工作幅度)
    /// 5. rated_load = 从载荷表查找(boom_length, working_radius)
    /// 6. moment_percentage = (current_load * working_radius) / (rated_load * working_radius)
    pub fn from_sensor_data_with_config(
        raw_data: SensorData,
        config: &CraneConfig,
        sequence_number: u64,
    ) -> Self {
        // ad1 -> current_load
        let current_load = config
            .sensor_calibration
            .convert_weight_ad_to_value(raw_data.ad1_load);

        // ad2 -> boom_length (臂长)
        let boom_length = config
            .sensor_calibration
            .convert_radius_ad_to_value(raw_data.ad2_radius);

        // ad3 -> boom_angle (臂角，0° = 水平)
        let boom_angle = config
            .sensor_calibration
            .convert_angle_ad_to_value(raw_data.ad3_angle);

        // 计算工作幅度: working_radius = boom_length * cos(boom_angle)
        let working_radius = Self::calculate_working_radius(boom_length, boom_angle);

        // 根据臂长和幅度查询额定载荷
        let rated_load = config
            .rated_load_table
            .get_rated_load(boom_length, working_radius);

        // 计算力矩百分比
        let moment_percentage =
            Self::calculate_moment_percentage_with_load(current_load, working_radius, rated_load);

        // 分别判断是否达到预警和报警阈值
        let is_warning = config.alarm_thresholds.is_moment_warning(moment_percentage);
        let is_danger = config.alarm_thresholds.is_moment_alarm(moment_percentage);

        // 检查传感器值是否超过预警/报警阈值
        let mut validation_error = None;

        if config.alarm_thresholds.is_moment_alarm(moment_percentage) {
            validation_error = Some(format!(
                "力矩报警: {:.1}% >= {:.1}%",
                moment_percentage, config.alarm_thresholds.moment.alarm_percentage
            ));
        } else if config.alarm_thresholds.is_moment_warning(moment_percentage) {
            validation_error = Some(format!(
                "力矩预警: {:.1}% >= {:.1}%",
                moment_percentage, config.alarm_thresholds.moment.warning_percentage
            ));
        }

        Self {
            current_load,
            rated_load,
            working_radius,
            boom_angle,
            boom_length,
            moment_percentage,
            is_warning,
            is_danger,
            validation_error,
            timestamp: SystemTime::now(),
            sequence_number,
            alarm_sources: Vec::new(),
            alarm_messages: Vec::new(),
        }
    }

    /// 计算工作幅度（working_radius）
    ///
    /// 公式: working_radius = boom_length * cos(boom_angle)
    ///
    /// - 0° = 吊臂水平（最大幅度）
    /// - 90° = 吊臂垂直（幅度为0）
    /// - >90° = 吊臂后仰，钳位到0
    fn calculate_working_radius(boom_length: f64, boom_angle_degrees: f64) -> f64 {
        let angle_rad = boom_angle_degrees.to_radians();
        let cos_angle = angle_rad.cos();
        // 钳位到0（吊臂后仰时幅度为0）
        let effective_cos = cos_angle.max(0.0);
        boom_length * effective_cos
    }

    /// 计算力矩百分比
    ///
    /// 力矩 = 载荷 × 工作半径
    /// 力矩百分比 = (当前力矩 / 额定力矩) × 100%
    fn calculate_moment_percentage(data: &SensorData) -> f64 {
        let current_moment = data.ad1_load * data.ad2_radius;
        let rated_moment = Self::DEFAULT_RATED_LOAD * data.ad2_radius;

        if rated_moment > 0.0 {
            ((current_moment / rated_moment) * 100.0).min(100.0)
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
            ((current_moment / rated_moment) * 100.0).min(100.0)
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_sensor_data() {
        let sensor_data = SensorData::new(20.0, 10.0, 60.0);
        let processed = ProcessedData::from_sensor_data(sensor_data, 1);

        assert_eq!(processed.sequence_number, 1);
        assert_eq!(processed.current_load, 20.0);
        assert_eq!(processed.working_radius, 10.0);
        assert_eq!(processed.boom_angle, 60.0);
        assert_eq!(processed.boom_length, 10.0); // 简化模式：臂长等于半径
        assert_eq!(processed.moment_percentage, 80.0);
        assert!(!processed.is_danger);
    }

    #[test]
    fn test_warning_detection() {
        // 90-100% triggers warning but not danger
        let sensor_data = SensorData::new(23.0, 10.0, 60.0); // 92%
        let processed = ProcessedData::from_sensor_data(sensor_data, 2);

        assert!(processed.is_warning); // 92% >= 90%
        assert!(!processed.is_danger); // 92% < 100%
        assert!(processed.moment_percentage >= 90.0);
    }

    #[test]
    fn test_danger_detection() {
        // >=100% triggers both warning and danger
        let sensor_data = SensorData::new(25.0, 10.0, 60.0); // 100%
        let processed = ProcessedData::from_sensor_data(sensor_data, 2);

        assert!(processed.is_warning); // 100% >= 90%
        assert!(processed.is_danger); // 100% >= 100%
        assert!(processed.moment_percentage >= 100.0);
    }

    #[test]
    fn test_validation_error() {
        let sensor_data = SensorData::new(-5.0, 10.0, 60.0); // Invalid negative load
        let processed = ProcessedData::from_sensor_data(sensor_data, 3);

        assert!(processed.validation_error.is_some());
        assert!(processed.validation_error.unwrap().contains("负值"));
    }

    #[test]
    fn test_from_sensor_data_with_config() {
        // 创建配置
        let config = CraneConfig::default();

        // 创建传感器数据（AD 值：中点，角度为0使工作半径=臂长）
        // ad3=0 表示臂角为0°（水平），此时 working_radius = boom_length
        let sensor_data = SensorData::new(2047.5, 2047.5, 0.0);

        // 使用配置处理数据
        let processed = ProcessedData::from_sensor_data_with_config(sensor_data, &config, 1);

        // 验证转换结果（默认配置：0-4095 AD -> 0-50吨, 0-20米, 0-90度）
        assert!((processed.current_load - 25.0).abs() < 0.5); // 约 25 吨
        assert!((processed.working_radius - 10.0).abs() < 0.5); // 约 10 米（臂角0°时等于臂长）
        assert!((processed.boom_angle - 0.0).abs() < 0.5); // 0 度
        assert_eq!(processed.sequence_number, 1);
    }

    #[test]
    fn test_from_sensor_data_with_config_uses_rated_load_table() {
        // 创建配置
        let config = CraneConfig::default();

        // 创建传感器数据，工作半径对应 5 米（AD 值约 1023.75）
        // ad3=0 使臂角为0°，此时 working_radius = boom_length
        // 根据默认载荷表，5 米对应 40 吨额定载荷
        let sensor_data = SensorData::new(2047.5, 1023.75, 0.0);

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
        // ad3=0 使臂角为0°，working_radius = boom_length = 10m
        // 额定载荷 25 吨
        // 需要载荷 = 25 * 0.9 = 22.5 吨
        // AD 值 = 22.5 / 50 * 4095 ≈ 1842.75
        let sensor_data = SensorData::new(1842.75, 2047.5, 0.0);

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
        // ad3=0 使臂角为0°，working_radius = boom_length = 10m
        // 额定载荷 25 吨
        // 需要载荷 = 25 吨
        // AD 值 = 25 / 50 * 4095 ≈ 2047.5
        let sensor_data = SensorData::new(2047.5, 2047.5, 0.0);

        let processed = ProcessedData::from_sensor_data_with_config(sensor_data, &config, 1);

        // 验证报警状态
        assert!(processed.is_danger);
        assert!(processed.validation_error.is_some());
        let error_msg = processed.validation_error.unwrap();
        assert!(error_msg.contains("力矩报警") || error_msg.contains("力矩预警"));
    }

    #[test]
    fn test_calculate_moment_percentage_with_load() {
        // 测试力矩百分比计算
        let percentage = ProcessedData::calculate_moment_percentage_with_load(20.0, 10.0, 25.0);
        assert_eq!(percentage, 80.0);

        let percentage = ProcessedData::calculate_moment_percentage_with_load(25.0, 10.0, 25.0);
        assert_eq!(percentage, 100.0);

        // 超过100%时应该被限制为100%
        let percentage = ProcessedData::calculate_moment_percentage_with_load(30.0, 10.0, 25.0);
        assert_eq!(percentage, 100.0);
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

    #[test]
    fn test_calculate_working_radius_horizontal() {
        // 0° = 水平，幅度 = 臂长
        let radius = ProcessedData::calculate_working_radius(10.0, 0.0);
        assert!((radius - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_working_radius_vertical() {
        // 90° = 垂直，幅度 = 0
        let radius = ProcessedData::calculate_working_radius(10.0, 90.0);
        assert!((radius - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_working_radius_45_degrees() {
        // 45°，cos(45°) ≈ 0.707
        let radius = ProcessedData::calculate_working_radius(10.0, 45.0);
        let expected = 10.0 * 0.70710678;
        assert!((radius - expected).abs() < 0.01);
    }

    #[test]
    fn test_calculate_working_radius_60_degrees() {
        // 60°，cos(60°) = 0.5
        let radius = ProcessedData::calculate_working_radius(10.0, 60.0);
        assert!((radius - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_working_radius_over_90_degrees() {
        // >90° = 后仰，钳位到 0
        let radius = ProcessedData::calculate_working_radius(10.0, 120.0);
        assert!((radius - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_working_radius_negative_clamped() {
        // cos() 为负值时钳位到 0
        let radius = ProcessedData::calculate_working_radius(10.0, 150.0);
        assert!((radius - 0.0).abs() < 0.001);
    }
}
