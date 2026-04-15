#[derive(Debug, Clone, PartialEq)]
pub struct SensorData {
    pub ad1_load: f64,
    pub ad2_radius: f64,
    pub ad3_angle: f64,
}

impl SensorData {
    pub fn new(ad1_load: f64, ad2_radius: f64, ad3_angle: f64) -> Self {
        Self {
            ad1_load,
            ad2_radius,
            ad3_angle,
        }
    }

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
    fn test_validate() {
        let valid_data = SensorData::new(17.0, 10.0, 62.7);
        assert!(valid_data.validate().is_ok());

        let invalid_load = SensorData::new(-5.0, 10.0, 60.0);
        assert!(invalid_load.validate().is_err());

        let invalid_angle = SensorData::new(17.0, 10.0, 95.0);
        assert!(invalid_angle.validate().is_err());
    }
}
