// 传感器数据源（使用随机数模拟）

use crate::models::SensorData;
use sensor_simulator::prelude::*;

/// 传感器数据源
pub struct SensorDataSource {
    /// AD1 模拟器 - 载荷传感器（随机数）
    ad1_simulator: SimulatedSensor,
    
    /// AD2 模拟器 - 半径传感器（随机数）
    ad2_simulator: SimulatedSensor,
    
    /// AD3 模拟器 - 角度传感器（随机数）
    ad3_simulator: SimulatedSensor,
}

impl SensorDataSource {
    /// 创建新的传感器数据源
    pub fn new() -> Self {
        // AD1: 载荷传感器 - 随机数 [10.0, 25.0] 吨
        let ad1_simulator = SimulatedSensor::new(
            SimulatorType::Random { min: 10.0, max: 25.0 }
        );
        
        // AD2: 半径传感器 - 随机数 [5.0, 15.0] 米
        let ad2_simulator = SimulatedSensor::new(
            SimulatorType::Random { min: 5.0, max: 15.0 }
        );
        
        // AD3: 角度传感器 - 随机数 [30.0, 80.0] 度
        let ad3_simulator = SimulatedSensor::new(
            SimulatorType::Random { min: 30.0, max: 80.0 }
        );
        
        Self {
            ad1_simulator,
            ad2_simulator,
            ad3_simulator,
        }
    }
    
    /// 读取传感器数据
    pub fn read_data(&self) -> Result<SensorData, String> {
        // 从三个模拟器读取数据
        let ad1_load = self.ad1_simulator.read()
            .map_err(|e| format!("AD1 读取失败: {:?}", e))?;
        
        let ad2_radius = self.ad2_simulator.read()
            .map_err(|e| format!("AD2 读取失败: {:?}", e))?;
        
        let ad3_angle = self.ad3_simulator.read()
            .map_err(|e| format!("AD3 读取失败: {:?}", e))?;
        
        Ok(SensorData::new(ad1_load, ad2_radius, ad3_angle))
    }
}

impl Default for SensorDataSource {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_read_data() {
        let source = SensorDataSource::new();
        let result = source.read_data();
        
        assert!(result.is_ok());
        
        let data = result.unwrap();
        // 验证数据在合理范围内
        assert!(data.ad1_load >= 10.0 && data.ad1_load <= 25.0);
        assert!(data.ad2_radius >= 5.0 && data.ad2_radius <= 15.0);
        assert!(data.ad3_angle >= 30.0 && data.ad3_angle <= 80.0);
    }
    
    #[test]
    fn test_multiple_reads() {
        let source = SensorDataSource::new();
        
        // 多次读取应该返回不同的随机值
        let data1 = source.read_data().unwrap();
        let data2 = source.read_data().unwrap();
        
        // 随机数生成器应该产生不同的值（虽然理论上可能相同，但概率极低）
        assert!(data1.ad1_load != data2.ad1_load || 
                data1.ad2_radius != data2.ad2_radius || 
                data1.ad3_angle != data2.ad3_angle);
    }
}
