// 传感器数据模型

/// 传感器数据（AD1, AD2, AD3 输入）
#[derive(Debug, Clone, PartialEq)]
pub struct SensorData {
    /// AD1 - 当前载荷（吨）
    pub ad1_load: f64,
    
    /// AD2 - 工作半径（米）
    pub ad2_radius: f64,
    
    /// AD3 - 吊臂角度（度）
    pub ad3_angle: f64,
    
    /// 额定载荷（吨）- 配置参数
    pub rated_load: f64,
    
    /// 臂长（米）- 配置参数
    pub boom_length: f64,
}

impl SensorData {
    /// 创建新的传感器数据
    pub fn new(ad1_load: f64, ad2_radius: f64, ad3_angle: f64) -> Self {
        Self {
            ad1_load,
            ad2_radius,
            ad3_angle,
            rated_load: 25.0,  // 默认额定载荷
            boom_length: 22.6, // 默认臂长
        }
    }
    
    /// 计算力矩百分比
    pub fn calculate_moment_percentage(&self) -> f64 {
        let current_moment = self.ad1_load * self.ad2_radius;
        let rated_moment = self.rated_load * self.ad2_radius;
        
        if rated_moment > 0.0 {
            (current_moment / rated_moment) * 100.0
        } else {
            0.0
        }
    }
    
    /// 验证数据有效性
    pub fn validate(&self) -> Result<(), String> {
        if self.ad1_load < 0.0 {
            return Err("AD1 载荷数据异常：负值".to_string());
        }
        if self.ad2_radius < 0.0 {
            return Err("AD2 半径数据异常：负值".to_string());
        }
        if self.ad3_angle < 0.0 || self.ad3_angle > 90.0 {
            return Err("AD3 角度数据异常：超出范围 [0, 90]".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_new_sensor_data() {
        let data = SensorData::new(17.0, 10.0, 62.7);
        assert_eq!(data.ad1_load, 17.0);
        assert_eq!(data.ad2_radius, 10.0);
        assert_eq!(data.ad3_angle, 62.7);
    }
    
    #[test]
    fn test_calculate_moment_percentage() {
        let data = SensorData::new(20.0, 10.0, 60.0);
        let percentage = data.calculate_moment_percentage();
        assert_eq!(percentage, 80.0); // (20*10) / (25*10) * 100
    }
    
    #[test]
    fn test_validate() {
        let valid_data = SensorData::new(17.0, 10.0, 62.7);
        assert!(valid_data.validate().is_ok());
        
        let invalid_load = SensorData::new(-5.0, 10.0, 60.0);
        assert!(invalid_load.validate().is_err());
        
        let invalid_angle = SensorData::new(17.0, 10.0, 95.0);
        assert!(invalid_angle.validate().is_err());
    }
}
