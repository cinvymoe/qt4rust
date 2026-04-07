// 参数校准视图意图

use qt_rust_demo::models::SensorData;

/// 参数校准视图意图
#[derive(Debug, Clone)]
pub enum CalibrationIntent {
    /// 传感器数据更新
    SensorDataUpdated(SensorData),
    
    /// 传感器断连
    SensorDisconnected,
    
    /// 传感器重连
    SensorReconnected,
    
    /// 清除错误信息
    ClearError,
}
