// Sensor Provider Trait

use crate::error::SensorResult;

/// 统一的传感器接口
pub trait SensorProvider {
    /// 读取传感器数据
    fn read(&self) -> SensorResult<f64>;

    /// 检查传感器是否已连接
    fn is_connected(&self) -> bool {
        true
    }

    /// 获取传感器名称
    fn name(&self) -> &str {
        "Unknown Sensor"
    }
}
